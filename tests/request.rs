#[cfg(test)]
mod tests {
    use mockito::{Matcher, ServerGuard};
    use oai_pmh::client::{
        Client,
        query::{GetRecordArgs, ListRecordsArgs},
    };

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

        let args = GetRecordArgs::new(identifier, metadata_prefix);
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

    #[test]
    fn test_list_records() {
        let metadata_prefix = "oai_dc";

        let mut server = mockito::Server::new();

        let mock = setup_mock_server(
            &mut server,
            "tests/fixtures/list_records.xml",
            vec![
                Matcher::UrlEncoded("verb".into(), "ListRecords".into()),
                Matcher::UrlEncoded("metadataPrefix".into(), metadata_prefix.into()),
            ],
        );

        let args = ListRecordsArgs::new(metadata_prefix);
        let client = Client::new(&server.url()).unwrap();

        // Test cases for records from first page
        let test_cases = [
            (
                "oai:archivesspace:/repositories/2/archival_objects/1",
                "Correspondence about art, 1974–2014",
            ),
            (
                "oai:archivesspace:/repositories/2/archival_objects/2",
                "Correspondence about Veterans Affairs appeals, 1945–2012",
            ),
            (
                "oai:archivesspace:/repositories/2/archival_objects/3",
                "Correspondence relating to family, 1922–1972, undated",
            ),
        ];

        for (idx, record) in client.list_records(args).unwrap().enumerate() {
            let record = record.unwrap();

            if idx < test_cases.len() {
                let (expected_id, expected_title) = test_cases[idx];
                assert_eq!(record.header.identifier, expected_id);
                assert!(record.metadata.contains(expected_title));
            }

            // Break immediately after verifying test cases
            // This keeps us on the first page and avoids resumption token request
            // Refer to the examples for use of resumption tokens
            if idx >= test_cases.len() - 1 {
                break;
            }
        }

        mock.assert();
    }
}
