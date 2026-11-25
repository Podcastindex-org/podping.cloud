//! Feed url filtering.
//!
//! A plain text file holds one regex per line.  Blank lines and lines starting with `#` are ignored.
//! Patterns are case-insensitive by default (use `(?-i)` inside a pattern to turn that off).  They are
//! evaluated top down and the first one that matches causes the podping to be dropped.
//!
//! The file is re-checked at most every `FILTER_RECHECK_INTERVAL` and reloaded when its modified time
//! changes, so patterns can be edited on a running server.  A missing file means "no filters".  If the
//! file exists but can't be read, the previously loaded patterns stay in effect.
//!
//! At startup `ensure_default_file` writes `DEFAULT_FILTERS` (databases/filters.txt in the repo) to the
//! configured path if nothing is there yet.  An existing file is never touched.

use regex::{Regex, RegexBuilder};
use std::fs;
use std::io::ErrorKind;
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant, SystemTime};


//Globals ----------------------------------------------------------------------------------------------------
pub const FILTER_FILE_DEFAULT: &str = "/data/filters.txt";
const FILTER_RECHECK_INTERVAL: Duration = Duration::from_secs(5);
pub const DEFAULT_FILTERS: &str = include_str!("../databases/filters.txt");

static GLOBAL_FILTER: OnceLock<UrlFilter> = OnceLock::new();


//Structs ----------------------------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Pattern {
    pub line: usize,
    pub source: String,
    regex: Regex,
}

#[derive(Debug, PartialEq)]
pub struct FilterHit {
    pub line: usize,
    pub pattern: String,
}

struct FilterState {
    patterns: Vec<Pattern>,
    mtime: Option<SystemTime>,
    last_check: Option<Instant>,
}

pub struct UrlFilter {
    path: String,
    recheck_interval: Duration,
    state: RwLock<FilterState>,
}


//Functions --------------------------------------------------------------------------------------------------

/// Parse filter file contents into compiled patterns.  Invalid regexes are logged and skipped.
pub fn parse_patterns(contents: &str) -> Vec<Pattern> {
    let mut patterns = Vec::new();
    for (idx, raw) in contents.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match RegexBuilder::new(line).case_insensitive(true).build() {
            Ok(regex) => patterns.push(Pattern { line: idx + 1, source: line.to_string(), regex }),
            Err(e) => eprintln!("  Filter: skipping invalid regex on line {}: [{}] - {}", idx + 1, line, e),
        }
    }
    patterns
}

impl UrlFilter {
    pub fn new(path: &str, recheck_interval: Duration) -> UrlFilter {
        UrlFilter {
            path: path.to_string(),
            recheck_interval,
            state: RwLock::new(FilterState { patterns: Vec::new(), mtime: None, last_check: None }),
        }
    }

    /// Reload the file if it's due for a check and its modified time has changed.
    fn refresh(&self) {
        {
            let state = self.state.read().unwrap_or_else(|e| e.into_inner());
            if let Some(last) = state.last_check {
                if last.elapsed() < self.recheck_interval {
                    return;
                }
            }
        }

        let mut state = self.state.write().unwrap_or_else(|e| e.into_inner());
        //Another thread may have refreshed while we waited for the lock
        if let Some(last) = state.last_check {
            if last.elapsed() < self.recheck_interval {
                return;
            }
        }
        state.last_check = Some(Instant::now());

        let mtime = match fs::metadata(&self.path) {
            Ok(meta) => meta.modified().ok(),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                if state.mtime.is_some() || !state.patterns.is_empty() {
                    println!("  Filter: [{}] removed, filtering disabled.", self.path);
                }
                state.patterns.clear();
                state.mtime = None;
                return;
            }
            Err(e) => {
                eprintln!("  Filter: can't stat [{}], keeping current patterns: {}", self.path, e);
                return;
            }
        };

        if mtime.is_some() && mtime == state.mtime {
            return;
        }

        match fs::read_to_string(&self.path) {
            Ok(contents) => {
                state.patterns = parse_patterns(&contents);
                state.mtime = mtime;
                println!("  Filter: loaded {} pattern(s) from [{}].", state.patterns.len(), self.path);
            }
            Err(e) => {
                eprintln!("  Filter: can't read [{}], keeping current patterns: {}", self.path, e);
            }
        }
    }

    /// Returns the first pattern that matches any of the candidate strings.
    pub fn check(&self, candidates: &[&str]) -> Option<FilterHit> {
        self.refresh();
        let state = self.state.read().unwrap_or_else(|e| e.into_inner());
        for pattern in state.patterns.iter() {
            if candidates.iter().any(|c| pattern.regex.is_match(c)) {
                return Some(FilterHit { line: pattern.line, pattern: pattern.source.clone() });
            }
        }
        None
    }
}

/// The filter file path: `PODPING_FILTER_FILE`, else `FILTER_FILE_DEFAULT`.
pub fn configured_path() -> String {
    std::env::var("PODPING_FILTER_FILE").unwrap_or_else(|_| FILTER_FILE_DEFAULT.to_string())
}

/// Write the default filters to `path` if no file exists there.  Returns true if it was created.
/// Never overwrites; `create_new` makes the existence check and the create a single atomic step.
pub fn ensure_default_file(path: &str) -> std::io::Result<bool> {
    use std::io::Write;
    match fs::OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut f) => {
            f.write_all(DEFAULT_FILTERS.as_bytes())?;
            Ok(true)
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(e),
    }
}

/// The process-wide filter, reading from `configured_path()`.
pub fn global() -> &'static UrlFilter {
    GLOBAL_FILTER.get_or_init(|| {
        let path = configured_path();
        println!("Filter file: [{}]", path);
        UrlFilter::new(&path, FILTER_RECHECK_INTERVAL)
    })
}


//Tests ------------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("podping_filter_test_{}_{}", std::process::id(), name));
        p
    }

    #[test]
    fn skips_comments_blanks_and_invalid() {
        let pats = parse_patterns("# comment\n\n  example\\.com  \n(unclosed\nfoo");
        let srcs: Vec<_> = pats.iter().map(|p| (p.line, p.source.as_str())).collect();
        assert_eq!(srcs, vec![(3, "example\\.com"), (5, "foo")]);
    }

    #[test]
    fn case_insensitive_by_default_and_overridable() {
        let path = temp_path("case");
        fs::write(&path, "spam\\.com\n(?-i)OnlyUpper").unwrap();
        let f = UrlFilter::new(path.to_str().unwrap(), Duration::ZERO);
        assert!(f.check(&["https://SPAM.com/feed.xml"]).is_some());
        assert!(f.check(&["https://x.com/OnlyUpper"]).is_some());
        assert!(f.check(&["https://x.com/onlyupper"]).is_none());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn first_match_wins_and_checks_all_candidates() {
        let path = temp_path("order");
        fs::write(&path, "nomatch\nexample\\.com/feed\nexample").unwrap();
        let f = UrlFilter::new(path.to_str().unwrap(), Duration::ZERO);
        let hit = f.check(&["https://example.com%2Ffeed", "https://example.com/feed"]).unwrap();
        assert_eq!(hit, FilterHit { line: 2, pattern: "example\\.com/feed".to_string() });
        assert!(f.check(&["https://other.org/rss"]).is_none());
        fs::remove_file(&path).ok();
    }

    #[test]
    fn default_filters_drop_loopback_and_example_com() {
        let pats = parse_patterns(DEFAULT_FILTERS);
        assert_eq!(pats.len(), 2, "every default pattern should compile");
        let dropped = |u: &str| pats.iter().any(|p| p.regex.is_match(u));
        for u in ["http://localhost/feed.xml", "https://LOCALHOST:8080/rss", "http://127.0.0.1", "http://127.5.0.1/rss",
                  "http://user:pw@127.0.0.1:3000/x", "http://127.0.0.1?x=1", "http://0.0.0.0:80/", "http://[::1]:8080/feed",
                  "https://example.com/feed.xml", "https://feeds.Example.COM/rss", "http://u@a.b.example.com:443?x",
                  "https://example.com./x"] {
            assert!(dropped(u), "should drop {}", u);
        }
        for u in ["https://localhost.example.org/feed", "https://mylocalhost/rss", "https://127.0.0.1.nip.io/x",
                  "https://example.com.evil.net/rss", "https://notexample.com/rss", "https://example.org/rss",
                  "https://foo.com/example.com/rss", "https://example.net/?u=http://localhost/", "https://feeds.podcastindex.org/rss"] {
            assert!(!dropped(u), "should keep {}", u);
        }
    }

    #[test]
    fn ensure_default_file_creates_but_never_overwrites() {
        let path = temp_path("ensure");
        fs::remove_file(&path).ok();
        let p = path.to_str().unwrap();
        assert!(ensure_default_file(p).unwrap());
        assert_eq!(fs::read_to_string(&path).unwrap(), DEFAULT_FILTERS);

        fs::write(&path, "# my own\n").unwrap();
        assert!(!ensure_default_file(p).unwrap());
        assert_eq!(fs::read_to_string(&path).unwrap(), "# my own\n");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn missing_file_means_no_filters_and_hot_reload_works() {
        let path = temp_path("reload");
        fs::remove_file(&path).ok();
        let f = UrlFilter::new(path.to_str().unwrap(), Duration::ZERO);
        assert!(f.check(&["https://bad.com/rss"]).is_none());

        fs::write(&path, "bad\\.com").unwrap();
        assert!(f.check(&["https://bad.com/rss"]).is_some());

        //Make sure the mtime actually changes on coarse-grained filesystems
        std::thread::sleep(Duration::from_millis(1100));
        fs::write(&path, "other\\.com").unwrap();
        assert!(f.check(&["https://bad.com/rss"]).is_none());
        assert!(f.check(&["https://other.com/rss"]).is_some());

        fs::remove_file(&path).unwrap();
        assert!(f.check(&["https://other.com/rss"]).is_none());
    }
}
