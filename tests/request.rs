#[cfg(test)]
mod tests {
    use mockito::Matcher;
    use oai_pmh::client::{Client, query::GetRecordArgs};

    #[test]
    fn test_get_record() {
        let identifier = "oai:archivesspace:/repositories/2/resources/2";
        let metadata_prefix = "oai_ead";

        let xml = std::fs::read_to_string("tests/fixtures/get_record.xml")
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
            .with_body(xml)
            .create();

        let args = GetRecordArgs {
            identifier: identifier.into(),
            metadata_prefix: "oai_ead".into(),
        };

        let client = Client::new(&server.url()).unwrap();
        let _ = client.get_record(args).unwrap();

        mock.assert();
    }
}
