# oai-pmh

Rust library for the [Open Archives Initiative Protocol for Metadata Harvesting](https://www.openarchives.org/OAI/openarchivesprotocol.html).

*The immediate need and focus is for a client only. Repository may come later.*

## Usage

The standard practice will be to:

- Create a client passing in the OAI endpoint
- Create query args as necessary
- Run the query

```rust
use anyhow::Result;
use oai_pmh::client::{Client, query::ListRecordsArgs};

fn main() -> Result<()> {
    let client = Client::new("https://demo.archivesspace.org/oai")?;

    let response = client.identify()?;
    println!("{:?}", response.payload);

    let args = ListRecordsArgs::new("oai_dc");
    for record in client.list_records(args)? {
        println!("{:?}", record);
    }

    Ok(())
}
```

Queries that support resumption tokens return an iterator, as in `client.list_records` in the example.

## Metadata

To provide flexibilty metadata is not parsed by this library. The OAI response metadata element/s are captured as strings. The expectation is you "bring your own parser" to handle whatever metadata format is supported by the server and requested via the client.

## Runnable Examples

```bash
# Use defaults (test.archivesspace.org, oai_dc, 10 records)
cargo run --example list_records

# Specify endpoint and metadata prefix
cargo run --example list_records https://test.archivesspace.org oai_dc

# Specify number of records to fetch
cargo run --example list_records https://test.archivesspace.org oai_dc 25

# Specify a different server and metadata format
cargo run --example list_records https://demo.archivesspace.org/oai oai_ead 5
```
