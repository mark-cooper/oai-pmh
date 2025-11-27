#[cfg(test)]
mod tests {
    use mockito::{Matcher, ServerGuard};
    use oai_pmh::client::{Client, query::GetRecordArgs};

    fn setup_mock_server(
        server: &mut ServerGuard,
        fixture: &str,
        query_matchers: Vec<Matcher>,
    ) -> mockito::Mock {
        let xml = std::fs::read_to_string(fixture).expect("Failed to load fixture");

        server
            .mock("GET", "/")
            .match_query(Matcher::AllOf(query_matchers))
            .with_status(200)
            .with_header("content-type", "text/xml")
            .with_body(xml)
            .create()
    }

    #[test]
    fn test_get_record() {
        let identifier = "oai:archivesspace:/repositories/2/resources/2";
        let metadata_prefix = "oai_ead";

        let mut server = mockito::Server::new();

        let mock = setup_mock_server(
            &mut server,
            "tests/fixtures/get_record.xml",
            vec![
                Matcher::UrlEncoded("verb".into(), "GetRecord".into()),
                Matcher::UrlEncoded("identifier".into(), identifier.into()),
                Matcher::UrlEncoded("metadataPrefix".into(), metadata_prefix.into()),
            ],
        );

        let args = GetRecordArgs {
            identifier: identifier.into(),
            metadata_prefix: metadata_prefix.into(),
        };

        let client = Client::new(&server.url()).unwrap();
        let _ = client.get_record(args).unwrap();

        mock.assert();
    }

    #[test]
    fn test_identify() {
        let mut server = mockito::Server::new();

        let mock = setup_mock_server(
            &mut server,
            "tests/fixtures/identify.xml",
            vec![Matcher::UrlEncoded("verb".into(), "Identify".into())],
        );

        let client = Client::new(&server.url()).unwrap();
        let _ = client.identify().unwrap();

        mock.assert();
    }
}
