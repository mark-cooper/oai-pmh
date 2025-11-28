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
        .unwrap_or(10); // Default to first 10 headers

    println!("Fetching first {} headers...\n", limit);

    for header in client.list_identifiers(args)? {
        match header {
            Ok(header) => {
                count += 1;
                println!("Header #{}", count);
                println!("\t{:<12} {}", "Identifier:", header.identifier);
                println!("\t{:<12} {}", "Datestamp:", header.datestamp);

                if let Some(status) = &header.status {
                    println!("\t{:<12} {}", "Status:", status);
                }

                println!();

                if count >= limit {
                    println!("Reached limit of {} headers", limit);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error fetching record: {}", e);
                break;
            }
        }
    }

    println!("Total headers processed: {}", count);

    Ok(())
}
