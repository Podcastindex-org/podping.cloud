use crate::{Context, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use rusqlite::{Connection};
use std::error::Error;
use std::fmt;
use std::net::{TcpStream};
use std::io::{Write};


#[derive(Debug)]
struct HydraError(String);
//##: Implement
impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}
impl Error for HydraError {}

const SQLITE_FILE: &str = "podping.db";

pub async fn ping(ctx: Context) -> Response {
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
            
            //Make sure it's a fqdn
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

            //TODO: Need queueing and error checking here
            match hive_notify(url_incoming) {
                Ok(result) => {
                    println!("  {}", result);
                },
                Err(e) => {
                    eprintln!("  Hive Error: {}", e);
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

pub struct Publisher {
    name: String
}

fn check_auth(authstring: &str) -> Result<String, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE)?;
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

fn hive_notify(url: &str) -> Result<String, Box<dyn Error>> {
    match TcpStream::connect("localhost:5000") {
        Ok(mut stream) => {
            stream.write(url.as_bytes()).unwrap();
            Ok("Written to Hive".to_string())
        },
        Err(e) => {
            eprintln!("Failed to connect to hive-writer agent: {}", e);
            return Err(Box::new(HydraError(format!("Failed to connect to hive-write").into())));
        }
    }
}