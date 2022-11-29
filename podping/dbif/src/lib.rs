use rusqlite::{params, Connection};
use std::error::Error;
use std::fmt;
use std::str::FromStr;


//Globals --------------------------------------------------------------------------------------------------------------
const SQLITE_FILE_AUTH: &str = "/data/auth.db";
const SQLITE_FILE_QUEUE: &str = "/data/queue.db";
const PING_BATCH_SIZE: u64 = 1000;


//Structs & Enums ------------------------------------------------------------------------------------------------------
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
            "update" => Ok(Reason::Update),
            "live" => Ok(Reason::Live),
            "liveend" => Ok(Reason::LiveEnd),
            _ => Ok(Reason::Update),
        }
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reason::Update => write!(f, "update"),
            Reason::Live => write!(f, "live"),
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
            "podcast" => Ok(Medium::Podcast),
            "podcastl" => Ok(Medium::PodcastL),
            "music" => Ok(Medium::Music),
            "musicl" => Ok(Medium::MusicL),
            "video" => Ok(Medium::Video),
            "videol" => Ok(Medium::VideoL),
            "film" => Ok(Medium::Film),
            "filml" => Ok(Medium::FilmL),
            "audiobook" => Ok(Medium::Audiobook),
            "audiobookl" => Ok(Medium::AudiobookL),
            "newsletter" => Ok(Medium::Newsletter),
            "newsletterl" => Ok(Medium::NewsletterL),
            "blog" => Ok(Medium::Blog),
            "blogl" => Ok(Medium::BlogL),
            _ => Ok(Medium::Podcast),
        }
    }
}

impl fmt::Display for Medium {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Medium::Podcast => write!(f, "podcast"),
            Medium::PodcastL => write!(f, "podcastl"),
            Medium::Music => write!(f, "music"),
            Medium::MusicL => write!(f, "musicl"),
            Medium::Video => write!(f, "video"),
            Medium::VideoL => write!(f, "videol"),
            Medium::Film => write!(f, "film"),
            Medium::FilmL => write!(f, "filml"),
            Medium::Audiobook => write!(f, "audiobook"),
            Medium::AudiobookL => write!(f, "audiobookl"),
            Medium::Newsletter => write!(f, "newsletter"),
            Medium::NewsletterL => write!(f, "newsletterl"),
            Medium::Blog => write!(f, "blog"),
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


//Functions ------------------------------------------------------------------------------------------------------------

//Connect to the database at the given file location
fn connect_to_database(filepath: &str) -> Result<Connection, Box<dyn Error>> {
    if let Ok(conn) = Connection::open(filepath) {
        Ok(conn)
    } else {
        return Err(
            Box::new(
                HydraError(format!("Could not open a database file at: [{}].", filepath).into())
            )
        );
    }
}

//Create or update database files if needed
pub fn create_databases() -> Result<bool, Box<dyn Error>> {

    //Create the publishers table
    let mut conn = connect_to_database(SQLITE_FILE_AUTH)?;
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS publishers (
             name text,
             authval text primary key
         )",
        [],
    ) {
        Ok(_) => {
            println!("Publishers table is ready.");
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(
                Box::new(
                    HydraError(format!("Failed to create database publishers table: [{}].", SQLITE_FILE_AUTH).into())
                )
            );
        }
    }

    //Create the queue table
    conn = connect_to_database(SQLITE_FILE_QUEUE)?;
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS queue (
             url text primary key,
             createdon integer,
             reason text,
             medium text,
             inflight bool
         )",
        [],
    ) {
        Ok(_) => {
            println!("Queue table is ready.");
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(
                Box::new(
                    HydraError(format!("Failed to create database queue table: [{}].", SQLITE_FILE_QUEUE).into())
                )
            );
        }
    }
}

//Returns a vector of Publisher structs from the auth db or an Error
pub fn get_publishers() -> Result<Vec<Publisher>, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_AUTH)?;
    let mut pubs: Vec<Publisher> = Vec::new();

    let mut stmt = conn.prepare("SELECT name \
                                   FROM publishers \
                                   ORDER BY rowid ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Publisher {
            name: row.get(0)?
        })
    }).unwrap();

    for pubrow in rows {
        let publisher: Publisher = pubrow.unwrap();
        pubs.push(publisher);
    }

    Ok(pubs)
}

//Returns a vector of Ping structs from the queue or an Error
pub fn get_pings_from_queue(with_in_flight: bool) -> Result<Vec<Ping>, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_QUEUE)?;
    let mut pings: Vec<Ping> = Vec::new();

    //With in flights also?
    let mut inflight_clause = "inflight = 0";
    if with_in_flight {
        inflight_clause = "inflight >= 0";
    }

    let sqltxt = format!("SELECT url,\
                               createdon, \
                               reason, \
                               medium \
                        FROM queue \
                        WHERE {} \
                          AND createdon < (STRFTIME('%s') - 15) \
                        ORDER BY reason ASC, \
                                  rowid ASC \
                        LIMIT {}", inflight_clause, PING_BATCH_SIZE);

    let mut stmt = conn.prepare(&sqltxt)?;
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
    let conn = connect_to_database(SQLITE_FILE_QUEUE)?;

    match conn.execute("INSERT INTO queue (url, createdon, reason, medium, inflight) \
                                   VALUES (?1,  ?2,        ?3,     ?4    , 0)",
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
            match ping.reason {
                Reason::Live | Reason::LiveEnd => {
                    return update_ping_in_queue(&ping);
                }
                _ => return Err(Box::new(HydraError(format!("URL already in queue: [{}].", ping.url).into())))
            }
        }
    }
}

//Change the info for a ping by it's url. Returns Ok(true/false) or an Error
pub fn update_ping_in_queue(ping: &Ping) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_QUEUE)?;


    match conn.execute("UPDATE queue \
                        SET inflight = 0, \
                            createdon = ?, \
                            reason = ?, \
                            medium = ? \
                        WHERE url = ?",
                       params![
                           ping.time,
                           ping.reason.to_string(),
                           ping.medium.to_string(),
                           ping.url,
                       ])
    {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Updating ping with new info failed: [{:#?}].", ping).into())));
        }
    }
}

//Marks a ping record as inflight. Returns Ok(true/false) or an Error
pub fn set_ping_as_inflight(ping: &Ping) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_QUEUE)?;

    match conn.execute("UPDATE queue \
                        SET inflight = 1 \
                        WHERE url = ?",
                       params![
                           ping.url
                       ])
    {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Marking ping as inflight failed: [{:#?}].", ping).into())));
        }
    }
}

//Adds a url to the queue. Takes a Ping struct as input. Returns Ok(true/false) or an Error
pub fn reset_pings_in_flight() -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_QUEUE)?;

    match conn.execute("UPDATE queue \
                        SET inflight = 0, \
                            createdon = STRFTIME('%s') \
                        WHERE inflight = 1 \
                          AND createdon < (STRFTIME('%s') - 180)\
                        LIMIT 25",
                       params![])
    {
        Ok(_) => {
            Ok(true)
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(Box::new(HydraError(format!("Filed to reset old, inflight pings").into())));
        }
    }
}

//Deletes a url from the queue. Takes a url as a String. Returns Ok(true/false) or an Error
pub fn delete_ping_from_queue(url: String) -> Result<bool, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_QUEUE)?;

    conn.execute(
        "DELETE FROM queue \
          WHERE url = ?1",
        params![url],
    )?;

    Ok(true)
}

//Returns the name of the publisher that corresponds with this authorization header or an Error
pub fn check_auth(authstring: &str) -> Result<String, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_AUTH)?;
    let mut tokens: Vec<Publisher> = Vec::new();

    let mut stmt = conn.prepare("SELECT name \
                                   FROM publishers \
                                  WHERE authval LIKE :authstring \
                                  LIMIT 1")?;
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
pub fn check_auth_hybrid(authstring: &str) -> Result<String, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_AUTH)?;
    let mut tokens: Vec<Publisher> = Vec::new();

    let authstringparm = &authstring[0..22];

    println!("{}", authstringparm);

    let mut stmt = conn.prepare("SELECT name \
                                   FROM publishers \
                                  WHERE authval LIKE :authstring||'%' \
                                  LIMIT 1")?;
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