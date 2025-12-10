use anyhow::Result;
use oai_pmh::{Client, ListMetadataFormatsArgs};

fn main() -> Result<()> {
    let client = Client::new("https://demo.archivesspace.org/oai")?;

    let format_names: Vec<String> = client
        .list_metadata_formats(None::<ListMetadataFormatsArgs>)?
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

    let set_names: Vec<String> = client
        .list_sets()?
        .flat_map(|response| response.unwrap().payload.unwrap().set)
        .map(|set| set.set_name)
        .collect();

    println!("Found {} sets:\n", set_names.len());
    for name in &set_names {
        println!("\t{}", name);
    }

    Ok(())
}
