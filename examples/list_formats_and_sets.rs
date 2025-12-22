use oai_pmh::{Client, ListMetadataFormatsArgs, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("https://demo.archivesspace.org/oai")?;

    let format_names: Vec<String> = client
        .list_metadata_formats(None::<ListMetadataFormatsArgs>)
        .await?
        .payload
        .unwrap()
        .metadata_format
        .iter()
        .map(|format| format.metadata_prefix.clone())
        .collect();

    println!("Found {} formats:\n", format_names.len());
    for name in &format_names {
        println!("\t{}", name);
    }

    println!();

    let mut set_names: Vec<String> = Vec::new();
    let mut stream = client.list_sets().await?;
    while let Some(response) = stream.next().await {
        let response = response?;
        if let Some(payload) = response.payload {
            for set in payload.set {
                set_names.push(set.set_name);
            }
        }
    }

    println!("Found {} sets:\n", set_names.len());
    for name in &set_names {
        println!("\t{}", name);
    }

    Ok(())
}
