#[cfg(test)]
mod tests {
    use mockito::{Matcher, ServerGuard};
    use oai_pmh::client::{
        Client,
        query::{GetRecordArgs, ListIdentifiersArgs, ListMetadataFormatsArgs, ListRecordsArgs},
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
    fn test_list_identifiers() {
        let metadata_prefix = "oai_ead";

        let mut server = mockito::Server::new();

        let mock = setup_mock_server(
            &mut server,
            "tests/fixtures/list_identifiers.xml",
            vec![
                Matcher::UrlEncoded("verb".into(), "ListIdentifiers".into()),
                Matcher::UrlEncoded("metadataPrefix".into(), metadata_prefix.into()),
            ],
        );

        let args = ListIdentifiersArgs::new(metadata_prefix);
        let client = Client::new(&server.url()).unwrap();

        // Test cases for headers from first page
        let test_cases = [(
            "oai:archivesspace:/repositories/2/resources/2",
            "2025-11-11T14:28:08Z",
        )];

        for response in client.list_identifiers(args).unwrap() {
            let response = response.unwrap();
            let headers = response.payload.unwrap().header;

            for idx in 0..test_cases.len() {
                let header = &headers[idx];
                let (expected_id, expected_datestamp) = test_cases[idx];
                assert_eq!(header.identifier, expected_id);
                assert_eq!(header.datestamp, expected_datestamp);
            }

            break; // No pagination
        }

        mock.assert();
    }

    #[test]
    fn test_list_metadata_formats() {
        let mut server = mockito::Server::new();

        let mock = setup_mock_server(
            &mut server,
            "tests/fixtures/list_metadata_formats.xml",
            vec![Matcher::UrlEncoded(
                "verb".into(),
                "ListMetadataFormats".into(),
            )],
        );

        let args = None::<ListMetadataFormatsArgs>;
        let client = Client::new(&server.url()).unwrap();

        let test_cases = [
            (
                "oai_dc",
                "http://www.openarchives.org/OAI/2.0/oai_dc.xsd",
                "http://www.openarchives.org/OAI/2.0/oai_dc/",
            ),
            (
                "oai_ead",
                "https://www.loc.gov/ead/ead.xsd",
                "http://www.loc.gov/ead/",
            ),
        ];

        let response = client.list_metadata_formats(args).unwrap();
        let metadata_formats = response.payload.unwrap().metadata_format;

        for idx in 0..test_cases.len() {
            let format = &metadata_formats[idx];
            let (expected_prefix, expected_schema, expected_namespace) = test_cases[idx];
            assert_eq!(format.metadata_prefix, expected_prefix);
            assert_eq!(format.schema, expected_schema);
            assert_eq!(format.metadata_namespace, expected_namespace);
        }

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

        for response in client.list_records(args).unwrap() {
            let response = response.unwrap();
            let records = response.payload.unwrap().record;

            for idx in 0..test_cases.len() {
                let record = &records[idx];
                let (expected_id, expected_title) = test_cases[idx];
                assert_eq!(record.header.identifier, expected_id);
                assert!(record.metadata.contains(expected_title));
            }

            break; // No pagination
        }

        mock.assert();
    }
}
