#[cfg(test)]
mod tests {
    use mockito::Matcher;
    use oai_pmh::client::{Client, query::GetRecordArgs};

    #[test]
    fn test_get_record() {
        let identifier = "oai:archivesspace:/repositories/2/resources/2";
        let metadata_prefix = "oai_ead";

        let fixture = std::fs::read_to_string("tests/fixtures/get_record.ead.xml")
            .expect("Failed to load fixture");

        let mut server = mockito::Server::new();

        let mock = server
            .mock("GET", "/")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("verb".into(), "GetRecord".into()),
                Matcher::UrlEncoded("identifier".into(), identifier.into()),
                Matcher::UrlEncoded("metadataPrefix".into(), metadata_prefix.into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/xml")
            .with_body(fixture.to_string())
            .create();

        let args = GetRecordArgs {
            identifier: identifier.to_string(),
            metadata_prefix: "oai_ead".to_string(),
        };

        let client = Client::new(&server.url()).unwrap();
        let _ = client.get_record(args).unwrap();

        mock.assert();
    }
}
