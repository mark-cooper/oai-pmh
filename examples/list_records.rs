use anyhow::Result;
use oai_pmh::client::{Client, query::ListRecordsArgs};

fn main() -> Result<()> {
    // Default to test server, or use first arg
    let endpoint = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "https://test.archivesspace.org/oai".to_string());

    let metadata_prefix = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "oai_dc".to_string());

    println!("Connecting to: {}", endpoint);
    println!("Metadata prefix: {}", metadata_prefix);
    println!();

    let client = Client::new(&endpoint)?;
    let args = ListRecordsArgs::new(metadata_prefix);

    let mut count = 0;
    let limit = std::env::args()
        .nth(3)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10); // Default to first 10 records

    println!("Fetching first {} records...\n", limit);

    for record in client.list_records(args)? {
        match record {
            Ok(record) => {
                count += 1;
                println!("Record #{}", count);
                println!("\t{:<12} {}", "Identifier:", record.header.identifier);
                println!("\t{:<12} {}", "Datestamp:", record.header.datestamp);

                if let Some(status) = &record.header.status {
                    println!("\t{:<12} {}", "Status:", status);
                }

                // Show first 200 chars of metadata (or indicate if empty)
                if record.metadata.is_empty() {
                    println!("\t{:<12} {}", "Metadata:", "(empty)");
                } else {
                    let metadata_preview = if record.metadata.len() > 200 {
                        format!("{}...", &record.metadata[..200])
                    } else {
                        record.metadata.clone()
                    };
                    // Find first non-empty line
                    let first_line = metadata_preview
                        .lines()
                        .find(|line| !line.trim().is_empty())
                        .unwrap_or("(no content)");
                    println!("\t{:<12} {}", "Metadata:", first_line);
                }
                println!();

                if count >= limit {
                    println!("Reached limit of {} records", limit);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error fetching record: {}", e);
                break;
            }
        }
    }

    println!("Total records processed: {}", count);

    Ok(())
}
