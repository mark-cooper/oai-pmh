//! Harvest a list, backing off when the server is busy (HTTP 503 / 429).
//!
//! A failed `try_next()` keeps its resumption token, so after sleeping we can
//! simply call it again and resume the same page. The first request is made when
//! the stream is created, so it needs its own retry.

use oai_pmh::{Client, Error, ListIdentifiersArgs, Result};
use std::time::Duration;

const MAX_RETRIES: u32 = 5;
const DEFAULT_DELAY: Duration = Duration::from_secs(10);
const MAX_DELAY: Duration = Duration::from_secs(300);

#[tokio::main]
async fn main() -> Result<()> {
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://test.archivesspace.org/oai".to_string());
    let metadata_prefix = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "oai_ead".to_string());

    let client = Client::new(&endpoint)?;

    let mut retries = 0;
    let mut stream = loop {
        let args = ListIdentifiersArgs::new(metadata_prefix.clone());
        match client.list_identifiers(args).await {
            Ok(stream) => break stream,
            Err(Error::RateLimited { status, retry_after }) if retries < MAX_RETRIES => {
                retries += 1;
                backoff(status, retry_after, retries).await;
            }
            Err(err) => return Err(err),
        }
    };

    let mut pages = 0;
    retries = 0;

    loop {
        match stream.try_next().await {
            Ok(Some(response)) => {
                retries = 0;
                pages += 1;
                let count = response.payload.map_or(0, |p| p.header.len());
                println!("Page #{pages}: {count} identifiers");
            }
            Ok(None) => break,
            Err(Error::RateLimited { status, retry_after }) if retries < MAX_RETRIES => {
                retries += 1;
                backoff(status, retry_after, retries).await;
            }
            Err(err) => return Err(err),
        }
    }

    println!("Done: {pages} pages");
    Ok(())
}

/// Honor the server's Retry-After (seconds), falling back to a default.
async fn backoff(status: u16, retry_after: Option<Duration>, attempt: u32) {
    let delay = retry_after.unwrap_or(DEFAULT_DELAY).min(MAX_DELAY);
    eprintln!(
        "HTTP {status}: retry {attempt}/{MAX_RETRIES} in {}s",
        delay.as_secs()
    );
    tokio::time::sleep(delay).await;
}
