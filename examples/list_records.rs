use oai_pmh::{Client, ListRecordsArgs, Result};

#[tokio::main]
async fn main() -> Result<()> {
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
        .unwrap_or(5); // Default to first 5 responses

    println!("Fetching first {} responses...\n", limit);

    let mut stream = client.list_records(args).await?;
    while let Some(response) = stream.next().await {
        count += 1;
        println!("Response #{}\n", count);

        match response {
            Ok(response) => {
                if let Some(error) = response.error {
                    eprintln!("\t{:<12} {}\n", "OAI-PMH error:", error);
                    continue;
                }

                if let Some(payload) = response.payload {
                    for record in payload.record {
                        println!("\t{:<12} {}", "Identifier:", record.header.identifier);
                        println!("\t{:<12} {}", "Datestamp:", record.header.datestamp);

                        if let Some(status) = &record.header.status {
                            println!("\t{:<12} {}", "Status:", status);
                        }

                        // Show first 50 chars of metadata (or indicate if empty)
                        if record.metadata.is_empty() {
                            println!("\t{:<12} {}", "Metadata:", "(empty)");
                        } else {
                            let metadata_preview = if record.metadata.len() > 50 {
                                format!("{}...", &record.metadata[..50])
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
