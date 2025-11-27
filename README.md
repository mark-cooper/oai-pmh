# oai-pmh

Rust library for the [Open Archives Initiative Protocol for Metadata Harvesting](https://www.openarchives.org/OAI/openarchivesprotocol.html).

*The immediate need and focus is for a client only. Repository may come later.*

## Examples

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
