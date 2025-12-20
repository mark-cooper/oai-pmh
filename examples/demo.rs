use oai_pmh::{Client, ListRecordsArgs, Result};

fn main() -> Result<()> {
    let client = Client::new("https://demo.archivesspace.org/oai")?;

    let response = client.identify()?;
    println!("{:?}", response.payload);

    let args = ListRecordsArgs::new("oai_dc");
    for response in client.list_records(args)? {
        println!("{:?}", response);
        break;
    }

    Ok(())
}
