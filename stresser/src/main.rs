// Based on code from: http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust
//
// Change the header auth token and set the url to your test location.  Please do not test against
// the live, public system.  The urls.txt is just one url per line.
//

use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use futures::stream::StreamExt;

const AUTH_HEADER_TOKEN: &str = "Blahblah^^12345678";
const PODPING_URL: &str = "http://localhost/?url=";
const MAX_CONCURRENT: usize = 8;


fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(
        reader.lines().filter_map(Result::ok).collect()
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths: Vec<String> = read_lines("urls.txt")?;

    let fetches = futures::stream::iter(
    paths.into_iter().map(|path| {
        async move {
            //Build the request url
            let mut pp_get_url: String = String::new();
            pp_get_url.push_str(PODPING_URL);
            pp_get_url.push_str(path.as_str());
            let client = reqwest::Client::new();
            match client.get(&pp_get_url)
              .header("Authorization", AUTH_HEADER_TOKEN)
              .header("User-Agent", "Stresser")
              .send()
              .await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok(text) => {
                            println!("RESPONSE: {} bytes from {}", text, pp_get_url);
                        }
                        Err(_) => println!("ERROR reading {}", pp_get_url),
                    }
                }
                Err(_) => println!("ERROR downloading {}", pp_get_url),
            }
        }
    })
    ).buffer_unordered(MAX_CONCURRENT).collect::<Vec<()>>();
    fetches.await;
    Ok(())
}