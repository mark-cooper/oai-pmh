use std::process;

use oai_pmh::{Client, Result};

fn main() -> Result<()> {
    let client = Client::new("https://demo.archivesspace.org/oai")?;

    let response = client.identify()?;

    println!("Response:\n");

    if let Some(error) = response.error {
        eprintln!("\t{:<12} {}\n", "OAI-PMH error:", error);
        process::exit(1);
    }

    if let Some(payload) = response.payload {
        println!("\t{:<12} {}", "Name:", payload.repository_name);
        println!("\t{:<12} {}", "Base Url:", payload.base_url);
        println!("\t{:<12} {:?}", "Email:", payload.admin_email);
    }

    Ok(())
}
