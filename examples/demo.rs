use oai_pmh::{Client, ListRecordsArgs, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("https://demo.archivesspace.org/oai")?;

    let response = client.identify().await?;
    println!("{:?}", response.payload);

    let args = ListRecordsArgs::new("oai_dc");
    let mut stream = client.list_records(args).await?;
    while let Some(response) = stream.next().await {
        println!("{:?}", response);
        break;
    }

    Ok(())
}
