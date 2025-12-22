use oai_pmh::{Client, Error};

#[tokio::main]
async fn main() {
    let client = match Client::new("https://example.org/oai") {
        Ok(c) => c,
        Err(Error::InvalidEndpoint(msg)) => {
            eprintln!("Bad endpoint: {msg}");
            return;
        }
        Err(e) => {
            eprintln!("Failed to create client: {e}");
            return;
        }
    };

    match client.identify().await {
        Ok(response) => println!("{:?}", response.payload),
        Err(Error::Http(e)) => eprintln!("Network error: {e}"),
        Err(Error::XmlParse(e)) => eprintln!("Invalid XML response: {e}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
