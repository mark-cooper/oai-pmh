#[cfg(test)]
mod tests {
    use mockito::{Matcher, ServerGuard};
    use oai_pmh::client::{
        Client,
        query::{GetRecordArgs, ListIdentifiersArgs, ListMetadataFormatsArgs, ListRecordsArgs},
    };
    use std::fs;

    fn setup_mock_server(
        server: &mut ServerGuard,
        fixture: &str,
        query_matchers: Vec<Matcher>,
    ) -> mockito::Mock {
        let xml = fs::read_to_string(fixture).expect("Failed to load fixture");

        server
            .mock("GET", "/")
            .match_query(Matcher::AllOf(query_matchers))
            .with_status(200)
            .with_header("content-type", "text/xml")
            .with_body(xml)
            .create()
    }

    #[tokio::test]
    async fn test_get_record() {
        let identifier = "oai:archivesspace:/repositories/2/resources/2";
        let metadata_prefix = "oai_ead";

        let mut server = mockito::Server::new_async().await;

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
        let _ = client.get_record(args).await.unwrap();

        mock.assert();
    }

    #[tokio::test]
    async fn test_identify() {
        let mut server = mockito::Server::new_async().await;

        let mock = setup_mock_server(
            &mut server,
            "tests/fixtures/identify.xml",
            vec![Matcher::UrlEncoded("verb".into(), "Identify".into())],
        );

        let client = Client::new(&server.url()).unwrap();
        let _ = client.identify().await.unwrap();

        mock.assert();
    }

    #[tokio::test]
    async fn test_list_identifiers() {
        let metadata_prefix = "oai_ead";

        let mut server = mockito::Server::new_async().await;

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

        let mut stream = client.list_identifiers(args).await.unwrap();
        if let Some(response) = stream.next().await {
            let response = response.unwrap();
            let headers = response.payload.unwrap().header;

            for idx in 0..test_cases.len() {
                let header = &headers[idx];
                let (expected_id, expected_datestamp) = test_cases[idx];
                assert_eq!(header.identifier, expected_id);
                assert_eq!(header.datestamp, expected_datestamp);
            }
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_list_metadata_formats() {
        let mut server = mockito::Server::new_async().await;

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

        let response = client.list_metadata_formats(args).await.unwrap();
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

    #[tokio::test]
    async fn test_custom_request_headers() {
        let mut server = mockito::Server::new_async().await;

        let xml =
            fs::read_to_string("tests/fixtures/identify.xml").expect("Failed to load fixture");

        let mock = server
            .mock("GET", "/")
            .match_query(Matcher::UrlEncoded("verb".into(), "Identify".into()))
            .match_header("x-api-key", "secret")
            .with_status(200)
            .with_header("content-type", "text/xml")
            .with_body(xml)
            .create_async()
            .await;

        let mut client = Client::new(&server.url()).unwrap();
        client.add_header("x-api-key", "secret").unwrap();

        let _ = client.identify().await.unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_custom_headers_override_defaults() {
        let mut server = mockito::Server::new_async().await;

        let xml =
            fs::read_to_string("tests/fixtures/identify.xml").expect("Failed to load fixture");

        let mock = server
            .mock("GET", "/")
            .match_query(Matcher::UrlEncoded("verb".into(), "Identify".into()))
            .match_header("user-agent", "custom-agent/1.0")
            .with_status(200)
            .with_header("content-type", "text/xml")
            .with_body(xml)
            .create_async()
            .await;

        let mut client = Client::new(&server.url()).unwrap();
        client.add_header("user-agent", "custom-agent/1.0").unwrap();

        let _ = client.identify().await.unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_records() {
        let metadata_prefix = "oai_dc";

        let mut server = mockito::Server::new_async().await;

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

        let mut stream = client.list_records(args).await.unwrap();
        if let Some(response) = stream.next().await {
            let response = response.unwrap();
            let records = response.payload.unwrap().record;

            for idx in 0..test_cases.len() {
                let record = &records[idx];
                let (expected_id, expected_title) = test_cases[idx];
                assert_eq!(record.header.identifier, expected_id);
                assert!(record.metadata.contains(expected_title));
            }
        }

        mock.assert();
    }

    #[tokio::test]
    async fn test_rate_limited_with_retry_after() {
        use oai_pmh::error::Error;
        use std::time::Duration;

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_query(Matcher::Any)
            .with_status(503)
            .with_header("retry-after", "30")
            .create();

        let client = Client::new(&server.url()).unwrap();
        let err = client.identify().await.unwrap_err();
        assert!(matches!(
            err,
            Error::RateLimited { status: 503, retry_after: Some(d) } if d == Duration::from_secs(30)
        ));
        mock.assert();
    }

    #[tokio::test]
    async fn test_rate_limited_429_and_http_date_retry_after() {
        use oai_pmh::error::Error;

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_query(Matcher::Any)
            .with_status(429)
            .with_header("retry-after", "Wed, 21 Oct 2026 07:28:00 GMT")
            .create();

        let client = Client::new(&server.url()).unwrap();
        let err = client.identify().await.unwrap_err();
        assert!(matches!(
            err,
            Error::RateLimited { status: 429, retry_after: None }
        ));
        mock.assert();
    }

    #[tokio::test]
    async fn test_stream_resumes_same_page_after_rate_limit() {
        use oai_pmh::error::Error;

        let mut server = mockito::Server::new_async().await;

        // Page 1 (has a resumption token)
        let page1 = setup_mock_server(
            &mut server,
            "tests/fixtures/list_identifiers.xml",
            vec![Matcher::UrlEncoded("metadataPrefix".into(), "oai_ead".into())],
        );

        // Page 2: busy once, then a final page without a token
        let busy = server
            .mock("GET", "/")
            .match_query(Matcher::Regex("resumptionToken=".into()))
            .with_status(503)
            .with_header("retry-after", "1")
            .expect(1)
            .create();
        let last = server
            .mock("GET", "/")
            .match_query(Matcher::Regex("resumptionToken=".into()))
            .with_status(200)
            .with_header("content-type", "text/xml")
            .with_body(
                "<OAI-PMH><responseDate>x</responseDate><request>y</request>\
                 <ListIdentifiers><header><identifier>last</identifier>\
                 <datestamp>d</datestamp></header></ListIdentifiers></OAI-PMH>",
            )
            .expect(1)
            .create();

        let client = Client::new(&server.url()).unwrap();
        let mut stream = client
            .list_identifiers(ListIdentifiersArgs::new("oai_ead"))
            .await
            .unwrap();

        assert!(stream.try_next().await.unwrap().is_some()); // page 1

        let err = stream.try_next().await.unwrap_err(); // page 2: busy
        assert!(matches!(err, Error::RateLimited { status: 503, .. }));

        // Retrying resumes the same page rather than ending the stream
        let page2 = stream.try_next().await.unwrap().unwrap();
        assert_eq!(page2.payload.unwrap().header[0].identifier, "last");
        assert!(stream.try_next().await.unwrap().is_none());

        page1.assert();
        busy.assert();
        last.assert();
    }

    #[tokio::test]
    async fn test_stream_errors_on_oai_error_in_continuation_page() {
        use oai_pmh::error::Error;

        let mut server = mockito::Server::new_async().await;

        let _page1 = setup_mock_server(
            &mut server,
            "tests/fixtures/list_identifiers.xml",
            vec![Matcher::UrlEncoded("metadataPrefix".into(), "oai_ead".into())],
        );
        let _page2 = server
            .mock("GET", "/")
            .match_query(Matcher::Regex("resumptionToken=".into()))
            .with_status(200)
            .with_header("content-type", "text/xml")
            .with_body(
                "<OAI-PMH><responseDate>x</responseDate><request>y</request>\
                 <error code=\"badResumptionToken\">expired</error></OAI-PMH>",
            )
            .create();

        let client = Client::new(&server.url()).unwrap();
        let mut stream = client
            .list_identifiers(ListIdentifiersArgs::new("oai_ead"))
            .await
            .unwrap();

        assert!(stream.try_next().await.unwrap().is_some());

        let err = stream.try_next().await.unwrap_err();
        match err {
            Error::Oai(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].message, "expired");
            }
            other => panic!("expected Error::Oai, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_stream_first_page_error_is_still_returned_as_response() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_header("content-type", "text/xml")
            .with_body(
                "<OAI-PMH><responseDate>x</responseDate><request>y</request>\
                 <error code=\"noRecordsMatch\"/></OAI-PMH>",
            )
            .create();

        let client = Client::new(&server.url()).unwrap();
        let mut stream = client
            .list_identifiers(ListIdentifiersArgs::new("oai_ead"))
            .await
            .unwrap();

        let response = stream.try_next().await.unwrap().unwrap();
        assert!(response.is_err());
        assert!(stream.try_next().await.unwrap().is_none());
    }
}
