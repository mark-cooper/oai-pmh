pub mod metadata;
pub mod query;
pub mod response;
pub mod resumable;

pub use resumable::ResumableStream;

use crate::Verb;
use crate::client::query::{
    GetRecordArgs, ListIdentifiersArgs, ListMetadataFormatsArgs, ListRecordsArgs, Query,
};
use crate::client::response::{
    GetRecordResponse, IdentifyResponse, ListIdentifiersResponse, ListMetadataFormatsResponse,
    ListRecordsResponse, ListSetsResponse,
};

use crate::error::{Error, Result};
use reqwest::header::{ACCEPT, HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use serde::Serialize;
use url::Url;

const ACCEPT_HEADER: &str = "application/xml, text/xml;q=0.9, */*;q=0.1";

pub struct Client {
    client: reqwest::Client,
    endpoint: Url,
    headers: HeaderMap,
}

impl Client {
    pub fn new(endpoint: &str) -> Result<Self> {
        let endpoint = Url::parse(endpoint)?;

        if !matches!(endpoint.scheme(), "http" | "https") {
            return Err(Error::InvalidEndpoint(format!(
                "Endpoint must be an http or https url, given: {endpoint}"
            )));
        }

        let client = Self {
            client: reqwest::Client::new(),
            endpoint,
            headers: HeaderMap::new(),
        };
        Ok(client)
    }

    pub fn add_header(&mut self, name: &str, value: &str) -> Result<()> {
        let header_name: HeaderName = name.parse()?;
        let header_value: HeaderValue = value.parse()?;
        self.headers.insert(header_name, header_value);
        Ok(())
    }

    pub async fn get_record(&self, args: GetRecordArgs) -> Result<GetRecordResponse> {
        let xml = self.do_query(Query::new(Verb::GetRecord, args)).await?;
        let response = GetRecordResponse::new(&xml)?;
        Ok(response)
    }

    pub async fn identify(&self) -> Result<IdentifyResponse> {
        let xml = self.do_query(Query::new(Verb::Identify, ())).await?;
        let response = IdentifyResponse::new(&xml)?;
        Ok(response)
    }

    pub async fn list_identifiers(
        &self,
        args: ListIdentifiersArgs,
    ) -> Result<ResumableStream<'_, ListIdentifiersResponse>> {
        ResumableStream::new(self, Verb::ListIdentifiers, args).await
    }

    pub async fn list_metadata_formats(
        &self,
        args: Option<ListMetadataFormatsArgs>,
    ) -> Result<ListMetadataFormatsResponse> {
        let xml = self
            .do_query(Query::new(Verb::ListMetadataFormats, args))
            .await?;
        let response = ListMetadataFormatsResponse::new(&xml)?;
        Ok(response)
    }

    pub async fn list_records(
        &self,
        args: ListRecordsArgs,
    ) -> Result<ResumableStream<'_, ListRecordsResponse>> {
        ResumableStream::new(self, Verb::ListRecords, args).await
    }

    pub async fn list_sets(&self) -> Result<ResumableStream<'_, ListSetsResponse>> {
        ResumableStream::new(self, Verb::ListSets, ()).await
    }

    fn build_url<T: Serialize>(&self, query: Query<T>) -> Result<Url> {
        let query = serde_qs::to_string(&query)?;
        let mut url = self.endpoint.clone();

        let combined_query = match url.query() {
            Some(existing) if !existing.is_empty() => format!("{existing}&{query}"),
            _ => query,
        };
        url.set_query(Some(&combined_query));

        Ok(url)
    }

    fn truncate_body(s: &str, max_chars: usize) -> String {
        let truncated: String = s.chars().take(max_chars).collect();
        if truncated.len() < s.len() {
            format!("{truncated}...")
        } else {
            truncated
        }
    }

    pub(crate) async fn do_query<T: Serialize>(&self, query: Query<T>) -> Result<String> {
        let url = self.build_url(query)?;
        let user_agent = format!("oai-pmh-rs/{}", env!("CARGO_PKG_VERSION"));

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, ACCEPT_HEADER.parse()?);
        headers.insert(USER_AGENT, user_agent.parse()?);
        headers.extend(self.headers.clone());

        let response = self.client.get(url).headers(headers).send().await?;

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let body = response.text().await?;

        if !status.is_success() {
            return Err(Error::UnexpectedResponse {
                content_type,
                body: format!("HTTP {status}: {}", Self::truncate_body(&body, 200)),
            });
        }

        // Heuristic: OAI-PMH should be XML with an <OAI-PMH> root element.
        // Don't require an XML declaration, since many servers omit it.
        let trimmed = body.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
        let looks_like_oai_pmh = trimmed.contains("<OAI-PMH") || trimmed.contains(":OAI-PMH");
        if !trimmed.starts_with('<') || !looks_like_oai_pmh {
            return Err(Error::UnexpectedResponse {
                content_type,
                body: Self::truncate_body(&body, 200),
            });
        }

        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use crate::Verb;
    use crate::client::Client;
    use crate::client::query::{
        GetRecordArgs, ListIdentifiersArgs, ListMetadataFormatsArgs, ListRecordsArgs, Query,
    };

    #[test]
    fn create_client_with_valid_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint);
        assert!(client.is_ok());
    }

    #[test]
    fn create_client_with_invalid_url() {
        let endpoints = vec![
            "test.archivesspace.org/oai",
            "ftp://test.archivesspace.org/oai",
        ];

        for endpoint in endpoints {
            let client = Client::new(endpoint);
            assert!(client.is_err());
        }
    }

    #[test]
    fn client_build_get_record_query_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(
            Verb::GetRecord,
            GetRecordArgs::new("oai:archivesspace:/repositories/2/resources/2", "oai_ead"),
        );
        let url = client.build_url(query).unwrap();

        assert_eq!(url.host_str(), Some("test.archivesspace.org"));
        assert_eq!(url.path(), "/oai");
        assert_eq!(
            url.query(),
            Some(
                "verb=GetRecord&identifier=oai:archivesspace:/repositories/2/resources/2&metadataPrefix=oai_ead"
            )
        );
    }

    #[test]
    fn client_build_identify_query_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(Verb::Identify, ());
        let url = client.build_url(query).unwrap();

        assert_eq!(url.host_str(), Some("test.archivesspace.org"));
        assert_eq!(url.path(), "/oai");
        assert_eq!(url.query(), Some("verb=Identify"));
    }

    #[test]
    fn client_build_list_identifiers_query_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(
            Verb::ListIdentifiers,
            ListIdentifiersArgs::new("oai_ead").set("speccol"),
        );
        let url = client.build_url(query).unwrap();

        assert_eq!(url.host_str(), Some("test.archivesspace.org"));
        assert_eq!(url.path(), "/oai");
        assert_eq!(
            url.query(),
            Some("verb=ListIdentifiers&metadataPrefix=oai_ead&set=speccol")
        );
    }

    #[test]
    fn client_build_list_metadata_formats_query_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(Verb::ListMetadataFormats, None::<ListMetadataFormatsArgs>);
        let url = client.build_url(query).unwrap();

        assert_eq!(url.host_str(), Some("test.archivesspace.org"));
        assert_eq!(url.path(), "/oai");
        assert_eq!(url.query(), Some("verb=ListMetadataFormats"));
    }

    #[test]
    fn client_build_list_records_query_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(Verb::ListRecords, ListRecordsArgs::new("oai_ead"));
        let url = client.build_url(query).unwrap();

        assert_eq!(url.host_str(), Some("test.archivesspace.org"));
        assert_eq!(url.path(), "/oai");
        assert_eq!(url.query(), Some("verb=ListRecords&metadataPrefix=oai_ead"));
    }

    #[test]
    fn client_build_list_sets_query_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(Verb::ListSets, ());
        let url = client.build_url(query).unwrap();

        assert_eq!(url.host_str(), Some("test.archivesspace.org"));
        assert_eq!(url.path(), "/oai");
        assert_eq!(url.query(), Some("verb=ListSets"));
    }

    #[test]
    fn client_build_query_url_with_existing_query() {
        let endpoint = "https://test.archivesspace.org/oai?foo=bar";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(Verb::Identify, ());
        let url = client.build_url(query).unwrap();

        assert_eq!(url.host_str(), Some("test.archivesspace.org"));
        assert_eq!(url.path(), "/oai");
        assert_eq!(url.query(), Some("foo=bar&verb=Identify"));
    }
}
