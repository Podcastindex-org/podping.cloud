use crate::{Context, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::time::{SystemTime};
use percent_encoding::percent_decode;
use std::str::FromStr;
use serde_json::json;
use handlebars::Handlebars;
use dbif::{Ping, Reason, Medium};



//Structs ----------------------------------------------------------------------------------------------------
#[derive(Debug)]
struct HydraError(String);

impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}

impl Error for HydraError {}



//Functions --------------------------------------------------------------------------------------------------
pub async fn ping(ctx: Context) -> Response {
    //Get a current timestamp
    let timestamp: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    //Prep a Ping struct to receive what we're getting
    let mut ping_in = Ping {
        url: "".to_string(),
        time: timestamp,
        reason: Reason::Update,
        medium: Medium::Podcast,
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
        }
        None => {
            println!("\nREQUEST: {}", ctx.state.remote_ip);
        }
    }

    //Give a landing page if no parameters were given
    if params.len() == 0 {
        let reg = Handlebars::new();
        let doc = fs::read_to_string("home.html").expect("Something went wrong reading the file.");
        let doc_rendered = reg.render_template(&doc, &json!({"version": ctx.state.version})).expect("Something went wrong rendering the file");
        return hyper::Response::builder()
            .status(StatusCode::OK)
            .body(format!("{}", doc_rendered).into())
            .unwrap();
    }

    //Check for a valid authorization header in the request
    match ctx.req.headers().get("authorization") {
        Some(auth_header) => {
            let authtest = dbif::check_auth(auth_header.to_str().unwrap());
            match authtest {
                Ok(authtest) => {
                    println!("  Publisher: {}", authtest);
                }
                Err(e) => {
                    eprintln!("  Publisher token not found: {}", e);
                    let authtest2 = dbif::check_auth_hybrid(auth_header.to_str().unwrap());
                    match authtest2 {
                        Ok(authtest2) => {
                            println!("  Publisher Hybrid: {}", authtest2);
                        }
                        Err(e) => {
                            eprintln!("  Hybrid token not found: {}", e);
                            return hyper::Response::builder()
                                .status(StatusCode::UNAUTHORIZED)
                                .body(format!("Bad Authorization header check").into())
                                .unwrap();
                        }
                    }
                }
            }
        }
        None => {
            return hyper::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(format!("Invalid Authorization header").into())
                .unwrap();
        }
    }

    //Check the user-agent
    match ctx.req.headers().get("user-agent") {
        Some(ua_string) => {
            println!("  User-Agent: {}", ua_string.to_str().unwrap());
        }
        None => {
            return hyper::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(format!("User-Agent header is required").into())
                .unwrap();
        }
    }

    //Check for a valid url parameter in the request
    //TODO: This should be a function call
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
                            .unwrap();
                    }
                }
                None => {
                    println!("Urls must contain a valid protocol schema prefix, like http:// or https://");
                    return hyper::Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(format!("Urls must contain a valid protocol schema prefix, like http:// or https://").into())
                        .unwrap();
                }
            }

            //Decode the url if it was percent encoded
            match percent_decode(url_incoming.as_bytes()).decode_utf8() {
                Ok(result_url) => {
                    println!("ResultUrl: {}", result_url);
                }
                Err(e) => {
                    eprintln!("ResultUrlError: {:#?}", e);
                }
            }

            //Add the url to the Ping we will be storing
            ping_in.url = url_incoming.clone();
        }
        None => {
            println!("Url parameter is missing.  Call as /?url=<podcast_url>");
            return hyper::Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("Url parameter is missing.  Call as /?url=<podcast_url>").into())
                .unwrap();
        }
    };

    //Check if a reason code exists
    if let Some(reason_incoming) = params.get("reason") {
        println!("  REASON: {}", reason_incoming);

        //Process the reason
        let reason_code = Reason::from_str(reason_incoming).unwrap();

        //Add the reason to the ping we will store
        ping_in.reason = reason_code;
    }

    //Check if a medium code exists
    if let Some(medium_incoming) = params.get("medium") {
        println!("  MEDIUM: {}", medium_incoming);

        //Process the reason
        let medium_code = Medium::from_str(medium_incoming).unwrap();

        //Add the reason to the ping we will store
        ping_in.medium = medium_code;
    }

    //Put the ping in the database
    match dbif::add_ping_to_queue(&ping_in) {
        Ok(_) => {
            println!("  Added: [{:#?}] to the queue.", ping_in);
            println!(" ");
        }
        Err(e) => {
            eprintln!("  Err: {:#?}", e);
        }
    }

    //Return success all the time so we don't burden the outside world with
    //our own internal struggles :-)
    return hyper::Response::builder()
        .status(StatusCode::OK)
        .body(format!("Success!").into())
        .unwrap();
}

pub async fn publishers(ctx: Context) -> Response {
    //println!("{:#?}", ctx);

    //Get the real IP of the connecting client
    match ctx.req.headers().get("cf-connecting-ip") {
        Some(remote_ip) => {
            println!("\nREQUEST[CloudFlare] - /publishers: {}", remote_ip.to_str().unwrap());
        }
        None => {
            println!("\nREQUEST - /publishers: {}", ctx.state.remote_ip);
        }
    }

    //Check the user-agent
    match ctx.req.headers().get("user-agent") {
        Some(ua_string) => {
            println!("  User-Agent: {}", ua_string.to_str().unwrap());
        }
        None => {
            return hyper::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(format!("User-Agent header is required").into())
                .unwrap();
        }
    }

    //Give back a page with a plain list of publishers
    let publist = dbif::get_publishers();
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
                .unwrap();
        }
        Err(e) => {
            eprintln!("Error getting publisher list: {}", e);
            return hyper::Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(format!("Error getting publishers list.").into())
                .unwrap();
        }
    }
}
