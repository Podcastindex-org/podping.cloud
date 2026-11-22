use rusqlite::{params, Connection};
use strum::{AsRefStr, Display, EnumString};
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

#[derive(Debug, Clone, Copy, AsRefStr, Display, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Reason {
    Update,
    Live,
    LiveEnd,
}

#[derive(Debug, Clone, Copy, AsRefStr, Display, EnumString)]
#[strum(ascii_case_insensitive)]
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
    Publisher,
    PublisherL,
    Course,
    CourseL
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

//Change the info for a ping by its url. Returns Ok(true/false) or an Error
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

//Number of leading characters of a hybrid authorization header that are matched against stored tokens
const HYBRID_PREFIX_LEN: usize = 22;

//Returns the name of the publisher that corresponds with this authorization header or an Error
pub fn check_auth(authstring: &str) -> Result<String, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_AUTH)?;
    find_publisher(&conn, authstring)
}

//Returns the name of the publisher that corresponds with this hybrid authorization header or an Error
pub fn check_auth_hybrid(authstring: &str) -> Result<String, Box<dyn Error>> {
    let conn = connect_to_database(SQLITE_FILE_AUTH)?;
    find_publisher_hybrid(&conn, authstring)
}

//Exact (ASCII case-insensitive) match of the whole header against a stored token.  This is a plain
//comparison, not LIKE, so wildcard characters in the header are not interpreted.
fn find_publisher(conn: &Connection, authstring: &str) -> Result<String, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT name \
                                   FROM publishers \
                                  WHERE authval = :authstring COLLATE NOCASE \
                                  LIMIT 1")?;
    let mut rows = stmt.query(&[(":authstring", authstring)])?;
    match rows.next()? {
        Some(row) => Ok(row.get(0)?),
        None => Err(Box::new(HydraError("No publisher match found for the given token.".into()))),
    }
}

//Matches the first HYBRID_PREFIX_LEN characters of the header against the start of a stored token.
//Headers shorter than that are rejected instead of panicking.
fn find_publisher_hybrid(conn: &Connection, authstring: &str) -> Result<String, Box<dyn Error>> {
    let prefix: String = authstring.chars().take(HYBRID_PREFIX_LEN).collect();
    if prefix.chars().count() < HYBRID_PREFIX_LEN {
        return Err(Box::new(HydraError(format!("Hybrid token must be at least {} characters.", HYBRID_PREFIX_LEN).into())));
    }

    let mut stmt = conn.prepare("SELECT name \
                                   FROM publishers \
                                  WHERE substr(authval, 1, :len) = :prefix COLLATE NOCASE \
                                  LIMIT 1")?;
    let mut rows = stmt.query(rusqlite::named_params! { ":len": HYBRID_PREFIX_LEN as i64, ":prefix": prefix })?;
    match rows.next()? {
        Some(row) => Ok(row.get(0)?),
        None => Err(Box::new(HydraError("No hybrid publisher match found for the given token.".into()))),
    }
}


//Tests ----------------------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "AbCdEfGhIjKlMnOpQrStUvWxYz0123456789";

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE publishers (name text, authval text primary key)", []).unwrap();
        conn.execute("INSERT INTO publishers (name, authval) VALUES ('Pub One', ?1)", [TOKEN]).unwrap();
        conn
    }

    #[test]
    fn exact_token_matches() {
        assert_eq!(find_publisher(&test_db(), TOKEN).unwrap(), "Pub One");
    }

    #[test]
    fn exact_match_is_ascii_case_insensitive_like_before() {
        assert_eq!(find_publisher(&test_db(), &TOKEN.to_lowercase()).unwrap(), "Pub One");
    }

    #[test]
    fn wildcards_do_not_bypass_exact_check() {
        let conn = test_db();
        for bad in ["%", "%%", "_", "AbCd%", &format!("{}_", &TOKEN[..TOKEN.len() - 1]), ""] {
            assert!(find_publisher(&conn, bad).is_err(), "should reject {:?}", bad);
        }
    }

    #[test]
    fn hybrid_matches_on_prefix() {
        let header = format!("{}SOMETHINGELSE", &TOKEN[..HYBRID_PREFIX_LEN]);
        assert_eq!(find_publisher_hybrid(&test_db(), &header).unwrap(), "Pub One");
    }

    #[test]
    fn hybrid_rejects_short_headers_without_panicking() {
        let conn = test_db();
        for bad in ["", "%", "short", &TOKEN[..HYBRID_PREFIX_LEN - 1], "ééééééééééé"] {
            assert!(find_publisher_hybrid(&conn, bad).is_err(), "should reject {:?}", bad);
        }
    }

    #[test]
    fn hybrid_wildcards_do_not_bypass() {
        let conn = test_db();
        for bad in ["%".repeat(HYBRID_PREFIX_LEN), "_".repeat(HYBRID_PREFIX_LEN), format!("AbCd{}", "%".repeat(HYBRID_PREFIX_LEN))] {
            assert!(find_publisher_hybrid(&conn, &bad).is_err(), "should reject {:?}", bad);
        }
    }
}