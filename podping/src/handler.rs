use crate::{Context, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use rusqlite::{params, Connection};
use std::error::Error;
use std::fmt;
use std::time::{SystemTime};
use percent_encoding::percent_decode;


//Globals ----------------------------------------------------------------------------------------------------
const SQLITE_FILE_AUTH: &str = "auth.db";
const SQLITE_FILE_QUEUE: &str = "queue.db";
const HTML_LANDING_PAGE: &str = "<!doctype html><meta charset=utf-8><head><title>Podping.cloud</title></head><body><center style='margin-top:100px;'><svg width='20mm' height='20mm' version='1.1' viewBox='0 0 260 260' xmlns='http://www.w3.org/2000/svg'> <g transform='translate(1021.5 843.78)'> <rect x='-1022.2' y='-843.76' width='260' height='260' fill='#aa0100'/> <g transform='rotate(30 1021.7 -6836.5)'> <path d='m2523-522.92-1.6328-111.21-97.477-54.198-95.844 57.016 1.6328 111.21 97.477 54.198z' fill='none' stroke='#fff' stroke-linecap='square' stroke-width='14.023'/> <path d='m2426.3-504.03a32.489 43.368 0 0 0-26.282 17.874l27.191 20.254 25.386-20.231a32.489 43.368 0 0 0-26.295-17.897z' fill='#fff'/> <g fill='none' stroke='#fff' stroke-linecap='square'> <path d='m2382.9-519.36c29.104-22.135 57.158-19.812 84.459 0.021' stroke-width='11.377'/> <path d='m2364.8-552.52c41.645-35.108 81.788-31.424 120.86 0.0332' stroke-width='15.081'/> <path d='m2335.4-585.72c61.865-56.473 121.5-50.547 179.53 0.0535' stroke-width='19.844'/> </g> </g> </g> </svg><h1 style='font-family: Arial, Helvetica, sans-serif;'>podping.cloud <small style='font-size:14px;'>(<a style='text-decoration:none;' href='https://github.com/Podcastindex-org/podping.cloud/blob/main/overviewandpurpose.md'>v0.1.8</a>)</small></h1><p>To see all new episode notifications follow the instructions <a href='https://github.com/Podcastindex-org/podping.cloud#simple-watcher-simple-watcherpy'>here</a>.</center></body></html>";


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

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    //println!("{:#?}", params);

    //Get the real IP of the connecting client
    match ctx.req.headers().get("cf-connecting-ip") {
        Some(remote_ip) => {
            println!("\nREQUEST[CloudFlare]: {}", remote_ip.to_str().unwrap()); 
        },
        None => {
            println!("\nREQUEST: {}", ctx.state.remote_ip);
        }
    }

    //Give a landing page if no parameters were given
    if params.len() == 0 {
        return hyper::Response::builder()
        .status(StatusCode::OK)
        .body(format!("{}", HTML_LANDING_PAGE).into())
        .unwrap();
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

    //Check the user-agent
    match ctx.req.headers().get("user-agent") {
        Some(ua_string) => {
            println!("  User-Agent: {}", ua_string.to_str().unwrap()); 
        },
        None => {
            return hyper::Response::builder()
              .status(StatusCode::UNAUTHORIZED)
              .body(format!("User-Agent header is required").into())
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
                  .unwrap();
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

            //Decode the url if it was percent encoded
            match percent_decode(url_incoming.as_bytes()).decode_utf8() {
                Ok(result_url) => {
                    println!("ResultUrl: {}", result_url);
                },
                Err(e) => {
                    eprintln!("ResultUrlError: {:#?}", e);
                }
            }

            //Queue the ping
            let ping_in = Ping {
                url: url_incoming.clone(),
                time: timestamp
            };
            let url = ping_in.url.clone();
            match add_ping_to_queue(ping_in) {
                Ok(_) => {
                    println!("  Added: [{}] to the queue.", url);
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

pub async fn publishers(ctx: Context) -> Response {
    //println!("{:#?}", ctx);

    //Get the real IP of the connecting client
    match ctx.req.headers().get("cf-connecting-ip") {
        Some(remote_ip) => {
            println!("\nREQUEST[CloudFlare] - /publishers: {}", remote_ip.to_str().unwrap()); 
        },
        None => {
            println!("\nREQUEST - /publishers: {}", ctx.state.remote_ip);
        }
    }

    //Check the user-agent
    match ctx.req.headers().get("user-agent") {
        Some(ua_string) => {
            println!("  User-Agent: {}", ua_string.to_str().unwrap()); 
        },
        None => {
            return hyper::Response::builder()
              .status(StatusCode::UNAUTHORIZED)
              .body(format!("User-Agent header is required").into())
              .unwrap()
        }
    }

    //Give back a page with a plain list of publishers
    let publist = get_publishers();
    match publist {
        Ok(publist) => {
            let mut htmlpage: String = String::new();
            for publisher in publist {
                htmlpage.push_str(publisher.name.as_str());
                htmlpage.push_str("\n");
            }
            return hyper::Response::builder()
            .status(StatusCode::OK)
            .body(format!("{}", htmlpage).into())
            .unwrap()
        },
        Err(e) => {
            eprintln!("Error getting publisher list: {}", e);
            return hyper::Response::builder()
              .status(StatusCode::NO_CONTENT)
              .body(format!("Error getting publishers list.").into())
              .unwrap()
        }
    }
}

//Returns a vector of Publisher structs from the auth db or an Error
pub fn get_publishers() -> Result<Vec<Publisher>, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_AUTH)?;
    let mut pubs: Vec<Publisher> = Vec::new();

    let mut stmt = conn.prepare("SELECT name FROM publishers ORDER BY rowid ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Publisher {
            name: row.get(0)?
        })
    }).unwrap();

    for pubrow in rows {
        let publisher: Publisher = pubrow.unwrap();
        //println!("  {:#?}", ping.url);
        pubs.push(publisher);
    }

    Ok(pubs)
}

//Returns a vector of Ping structs from the queue or an Error
pub fn get_pings_from_queue() -> Result<Vec<Ping>, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_QUEUE)?;
    let mut urls: Vec<Ping> = Vec::new();

    let mut stmt = conn.prepare("SELECT url,createdon FROM queue ORDER BY rowid ASC LIMIT 50")?;
    let rows = stmt.query_map([], |row| {
        Ok(Ping {
            url: row.get(0)?,
            time: row.get(1)?
        })
    }).unwrap();

    for urlrow in rows {
        let ping: Ping = urlrow.unwrap();
        //println!("  {:#?}", ping.url);
        urls.push(ping);
    }

    Ok(urls)
}

//Adds a url to the queue. Takes a Ping struct as input. Returns Ok(true/false) or an Error
pub fn add_ping_to_queue(ping: Ping) -> Result<bool, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_QUEUE)?;

    match conn.execute("INSERT INTO queue (url, createdon) VALUES (?1, ?2)", params![ping.url, ping.time]) {
        Ok(_) => {
            Ok(true)
        },
        Err(_e) => {
            // match e {
            //     Error::SqliteFailure(err, _) => {
            //         assert_eq!(err.code, ErrorCode::ConstraintViolation);
            //         check_extended_code(err.extended_code);
            //     }
            //     err => panic!("Unexpected error {}", err),
            // }
            return Err(Box::new(HydraError(format!("URL already in queue: [{}].", ping.url).into())));
        },
    }
}

//Deletes a url from the queue. Takes a url as a String. Returns Ok(true/false) or an Error
pub fn delete_ping_from_queue(url: String) -> Result<bool, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_QUEUE)?;

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
        return Err(Box::new(HydraError(format!("No publisher match found for: [{}].", authstring).into())));
    }

    Ok(tokens[0].name.clone())
}