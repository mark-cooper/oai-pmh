use anyhow::Result;
use oai_pmh::client::{Client, query::ListIdentifiersArgs};

fn main() -> Result<()> {
    // Default to test server, or use first arg
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://test.archivesspace.org/oai".to_string());

    let metadata_prefix = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "oai_ead".to_string());

    println!("Connecting to: {}", endpoint);
    println!("Metadata prefix: {}", metadata_prefix);
    println!();

    let client = Client::new(&endpoint)?;
    let args = ListIdentifiersArgs::new(metadata_prefix);

    let mut count = 0;
    let limit = std::env::args()
        .nth(3)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5); // Default to first 5 responses

    println!("Fetching first {} responses...\n", limit);

    for response in client.list_identifiers(args)? {
        count += 1;
        println!("Response #{}", count);

        match response {
            Ok(response) => {
                if let Some(error) = response.error {
                    eprintln!("\t{:<12} {}\n", "OAI-PMH error:", error);
                    continue;
                }

                if let Some(payload) = response.payload {
                    for header in payload.header {
                        println!("\t{:<12} {}", "Identifier:", header.identifier);
                        println!("\t{:<12} {}", "Datestamp:", header.datestamp);

                        if let Some(status) = header.status {
                            println!("\t{:<12} {}", "Status:", status);
                        }

                        println!();
                    }
                }

                if count >= limit {
                    break;
                }
            }
            Err(e) => {
                eprintln!("\t{:<12} {}\n", "Request error:", e);
                break;
            }
        }
    }

    println!("Total responses processed: {}", count);

    Ok(())
}
