use crate::{Context, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use rusqlite::{params, Connection};
use std::error::Error;
use std::fmt;
use std::net::{TcpStream};
use std::io::{Write};
use std::time::{SystemTime};


//Globals ----------------------------------------------------------------------------------------------------
const SQLITE_FILE_AUTH: &str = "auth.db";
const SQLITE_FILE_QUEUE: &str = "queue.db";


//Structs ----------------------------------------------------------------------------------------------------
#[derive(Debug)]
struct HydraError(String);
impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}
impl Error for HydraError {}
pub struct Publisher {
    pub name: String
}
pub struct Ping {
    pub url: String,
    pub time: u64
}


//Functions --------------------------------------------------------------------------------------------------
pub async fn ping(ctx: Context) -> Response {
    //Get a current timestamp
    let timestamp: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() - (86400 * 90),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    
    //println!("{:#?}", ctx);

    //TODO: Need user-agent logging

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    //println!("{:#?}", params);

    //Get the real IP of the connecting client
    match ctx.req.headers().get("cf-connecting-ip") {
        Some(remote_ip) => {
            println!("REQUEST[CloudFlare]: {}", remote_ip.to_str().unwrap()); 
        },
        None => {
            println!("REQUEST: {}", ctx.state.remote_ip);
        }
    }

    //Check for a valid authorization header in the request
    match ctx.req.headers().get("authorization") {
        Some(auth_header) => {
            let authtest = check_auth(auth_header.to_str().unwrap());
            match authtest {
                Ok(authtest) => {
                    println!("  Publisher: {}", authtest);
                },
                Err(e) => {
                    eprintln!("{}", e);
                    return hyper::Response::builder()
                      .status(StatusCode::UNAUTHORIZED)
                      .body(format!("Bad Authorization header check").into())
                      .unwrap()
                }
            }        
        },
        None => {
            return hyper::Response::builder()
              .status(StatusCode::UNAUTHORIZED)
              .body(format!("Invalid Authorization header").into())
              .unwrap()
        }
    }

    //Check for a valid url parameter in the request
    let url_incoming = params.get("url");
    match url_incoming {
        Some(url_incoming) => {
            println!("  URL: {}", url_incoming);

            //Make sure the url is not empty
            if url_incoming.len() == 0 {
                println!("    Url parameter is missing.  Call as /?url=<podcast_url>");
                return hyper::Response::builder()
                  .status(StatusCode::BAD_REQUEST)
                  .body(format!("Url parameter is missing.  Call as /?url=<podcast_url>").into())
                  .unwrap()
            }
            
            //Make sure it's an fqdn
            let proto_scheme_pos = url_incoming.to_lowercase().find("http");
            match proto_scheme_pos {
                Some(proto_scheme_pos) => {
                    if proto_scheme_pos != 0 {
                        println!("Urls must contain a valid protocol schema prefix, like http:// or https://");
                        return hyper::Response::builder()
                          .status(StatusCode::BAD_REQUEST)
                          .body(format!("Urls must contain a valid protocol schema prefix, like http:// or https://").into())
                          .unwrap()
                    }
                },
                None => {
                    println!("Urls must contain a valid protocol schema prefix, like http:// or https://");
                    return hyper::Response::builder()
                      .status(StatusCode::BAD_REQUEST)
                      .body(format!("Urls must contain a valid protocol schema prefix, like http:// or https://").into())
                      .unwrap()
                }
            }

            //TODO: Perhaps do a HEAD check on the url here to check for problems like 404, 500

            //Queue the ping
            let ping_in = Ping {
                url: url_incoming.clone(),
                time: timestamp
            };
            match add_ping_to_queue(ping_in) {
                Ok(_) => {
                    println!("  Added to the queue.");
                },
                Err(e) => {
                    eprintln!("  Err: {:#?}", e);
                }
            }

            println!(" ");
            return hyper::Response::builder()
              .status(StatusCode::OK)
              .body(format!("Success!").into())
              .unwrap()
        },
        None => {
            println!("Url parameter is missing.  Call as /?url=<podcast_url>");
            return hyper::Response::builder()
              .status(StatusCode::BAD_REQUEST)
              .body(format!("Url parameter is missing.  Call as /?url=<podcast_url>").into())
              .unwrap()
        }
    };

}

//Returns a vector of Ping structs from the queue or an Error
pub fn get_pings_from_queue() -> Result<Vec<Ping>, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_QUEUE)?;
    let mut urls: Vec<Ping> = Vec::new();

    let mut stmt = conn.prepare("SELECT url,createdon FROM queue ORDER BY rowid ASC LIMIT 10")?;
    let rows = stmt.query_map([], |row| {
        Ok(Ping {
            url: row.get(0)?,
            time: row.get(1)?
        })
    }).unwrap();

    for urlrow in rows {
        let ping: Ping = urlrow.unwrap();
        println!("{:#?}", ping.url);
        urls.push(ping);
    }

    Ok(urls)
}

//Adds a url to the queue. Takes a Ping struct as input. Returns Ok(true/false) or an Error
pub fn add_ping_to_queue(ping: Ping) -> Result<bool, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_QUEUE)?;

    conn.execute(
        "INSERT INTO queue (url, createdon) VALUES (?1, ?2)",
        params![ping.url, ping.time],
    )?;

    Ok(true)
}

//Deletes a url from the queue. Takes a url as a String. Returns Ok(true/false) or an Error
pub fn delete_ping_from_queue(url: String) -> Result<bool, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE)?;

    conn.execute(
        "DELETE FROM queue WHERE url = ?1",
        params![url],
    )?;

    Ok(true)
}

//Returns the name of the publisher that corresponds with this authorization header or an Error
fn check_auth(authstring: &str) -> Result<String, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_AUTH)?;
    let mut tokens: Vec<Publisher> = Vec::new();

    let mut stmt = conn.prepare("SELECT name FROM publishers WHERE authval = :authstring LIMIT 1")?;
    let rows = stmt.query_map(&[(":authstring", authstring)], |row| {
        Ok(Publisher {
            name: row.get(0)?
        })
    }).unwrap();

    for pubrow in rows {
        let publisher: Publisher = pubrow.unwrap();
        //println!("{}", publisher.name.clone());
        tokens.push(publisher);
    }

    if tokens.len() == 0 {
        return Err(Box::new(HydraError(format!("No publisher match found.").into())));
    }

    Ok(tokens[0].name.clone())
}

//Calls the hive-writer agent socket, sending a URL string and returns Ok if it worked, or an Error
pub fn hive_notify(url: &str) -> Result<String, Box<dyn Error>> {
    println!("Writing: [{}] to Hive...", url);
    match TcpStream::connect("localhost:9999") {
        Ok(mut stream) => {
            stream.write(url.as_bytes()).unwrap();
            Ok("OK".to_string())
        },
        Err(e) => {
            return Err(Box::new(HydraError(format!("Failed to connect to hive-write agent: {}", e).into())));
        }
    }
}