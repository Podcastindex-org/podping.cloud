use crate::{Context, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use rusqlite::{params, Connection};
use std::error::Error;
use std::fmt;
use std::fs;
use std::time::{SystemTime};
use percent_encoding::percent_decode;
use std::str::FromStr;


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
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Reason {
    Update,
    Live,
    LiveEnd,
}

impl FromStr for Reason {
    type Err = ();
    fn from_str(input: &str) -> Result<Reason, Self::Err> {
        match input.to_lowercase().as_str() {
            "update"  => Ok(Reason::Update),
            "live"    => Ok(Reason::Live),
            "liveend" => Ok(Reason::LiveEnd),
            _         => Ok(Reason::Update),
        }
    }
}
impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reason::Update  => write!(f, "update"),
            Reason::Live    => write!(f, "live"),
            Reason::LiveEnd => write!(f, "liveend"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Medium {
    Podcast,
    PodcastL,
    Music,
    MusicL,
    Video,
    VideoL,
    Film,
    FilmL,
    Audiobook,
    AudiobookL,
    Newsletter,
    NewsletterL,
    Blog,
    BlogL,
}
impl FromStr for Medium {
    type Err = ();
    fn from_str(input: &str) -> Result<Medium, Self::Err> {
        match input.to_lowercase().as_str() {
            "podcast"  => Ok(Medium::Podcast),
            "podcastl" => Ok(Medium::PodcastL),
            "music"    => Ok(Medium::Music),
            "musicl"   => Ok(Medium::MusicL),
            "video"    => Ok(Medium::Video),
            "videol"   => Ok(Medium::VideoL),
            "film"     => Ok(Medium::Film),
            "filml"    => Ok(Medium::FilmL),
            "audiobook"   => Ok(Medium::Audiobook),
            "audiobookl"  => Ok(Medium::AudiobookL),
            "newsletter"  => Ok(Medium::Newsletter),
            "newsletterl" => Ok(Medium::NewsletterL),
            "blog"  => Ok(Medium::Blog),
            "blogl" => Ok(Medium::BlogL),
            _       => Ok(Medium::Podcast),
        }
    }
}
impl fmt::Display for Medium {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Medium::Podcast  => write!(f, "podcast"),
            Medium::PodcastL => write!(f, "podcastl"),
            Medium::Music    => write!(f, "music"),
            Medium::MusicL   => write!(f, "musicl"),
            Medium::Video    => write!(f, "video"),
            Medium::VideoL   => write!(f, "videol"),
            Medium::Film     => write!(f, "film"),
            Medium::FilmL    => write!(f, "filml"),
            Medium::Audiobook   => write!(f, "audiobook"),
            Medium::AudiobookL  => write!(f, "audiobookl"),
            Medium::Newsletter  => write!(f, "newsletter"),
            Medium::NewsletterL => write!(f, "newsletterl"),
            Medium::Blog  => write!(f, "blog"),
            Medium::BlogL => write!(f, "blogl"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ping {
    pub url: String,
    pub time: u64,
    pub reason: Reason,
    pub medium: Medium,
}

#[derive(Debug, Clone)]
pub struct PingRow {
    pub url: String,
    pub time: u64,
    pub reason: String,
    pub medium: String,
}


//Functions --------------------------------------------------------------------------------------------------
pub async fn ping(ctx: Context) -> Response {
    //Get a current timestamp
    let timestamp: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() - (86400 * 90),
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
        let doc = fs::read_to_string("home.html").expect("Something went wrong reading the home page file.");
        return hyper::Response::builder()
            .status(StatusCode::OK)
            .body(format!("{}", doc).into())
            .unwrap();
    }

    //Check for a valid authorization header in the request
    match ctx.req.headers().get("authorization") {
        Some(auth_header) => {
            let authtest = check_auth(auth_header.to_str().unwrap());
            match authtest {
                Ok(authtest) => {
                    println!("  Publisher: {}", authtest);
                }
                Err(e) => {
                    eprintln!("  Publisher token not found: {}", e);
                    let authtest2 = check_auth_hybrid(auth_header.to_str().unwrap());
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
    match add_ping_to_queue(&ping_in) {
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
    let mut pings: Vec<Ping> = Vec::new();

    let mut stmt = conn.prepare("SELECT url,\
                                        createdon, \
                                        reason, \
                                        medium \
                                 FROM queue \
                                 ORDER BY rowid ASC \
                                 LIMIT 50")?;
    let rows = stmt.query_map([], |row| {
        Ok(PingRow {
            url: row.get(0)?,
            time: row.get(1)?,
            reason: row.get(2)?,
            medium: row.get(3)?,
        })
    }).unwrap();

    for row in rows {
        let pingrow = row.unwrap();
        let ping = Ping {
            url: pingrow.url,
            time: pingrow.time,
            reason: Reason::from_str(&pingrow.reason).unwrap(),
            medium: Medium::from_str(&pingrow.medium).unwrap(),
        };
        //println!("  {:#?}", ping.url);
        pings.push(ping);
    }

    Ok(pings)
}

//Adds a url to the queue. Takes a Ping struct as input. Returns Ok(true/false) or an Error
pub fn add_ping_to_queue(ping: &Ping) -> Result<bool, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_QUEUE)?;

    match conn.execute("INSERT INTO queue (url, createdon, reason, medium) \
                                   VALUES (?1,  ?2,        ?3,     ?4    )",
                       params![
                           ping.url,
                           ping.time,
                           ping.reason.to_string(),
                           ping.medium.to_string(),
                       ])
    {
        Ok(_) => {
            Ok(true)
        }
        Err(_e) => {
            // match e {
            //     Error::SqliteFailure(err, _) => {
            //         assert_eq!(err.code, ErrorCode::ConstraintViolation);
            //         check_extended_code(err.extended_code);
            //     }
            //     err => panic!("Unexpected error {}", err),
            // }
            return Err(Box::new(HydraError(format!("URL already in queue: [{}].", ping.url).into())));
        }
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

    let mut stmt = conn.prepare("SELECT name FROM publishers WHERE authval LIKE :authstring LIMIT 1")?;
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

//Returns the name of the publisher that corresponds with this hybrid authorization header or an Error
fn check_auth_hybrid(authstring: &str) -> Result<String, Box<dyn Error>> {
    let conn = Connection::open(SQLITE_FILE_AUTH)?;
    let mut tokens: Vec<Publisher> = Vec::new();

    let authstringparm = &authstring[0..22];

    println!("{}", authstringparm);

    let mut stmt = conn.prepare("SELECT name FROM publishers WHERE authval LIKE :authstring||'%' LIMIT 1")?;
    let rows = stmt.query_map(&[(":authstring", authstringparm)], |row| {
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
        return Err(Box::new(HydraError(format!("No hybrid publisher match found for: [{}].", authstringparm).into())));
    }

    Ok(tokens[0].name.clone())
}