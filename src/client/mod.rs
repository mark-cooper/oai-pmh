pub mod metadata;
pub mod query;
pub mod response;
pub(crate) mod resumable;

use crate::Verb;
use crate::client::query::{GetRecordArgs, ListIdentifiersArgs, ListRecordsArgs, Query};
use crate::client::response::{
    GetRecordResponse, IdentifyResponse, ListIdentifiersResponse, ListRecordsResponse,
};
use crate::client::resumable::ResumableIter;

use anyhow::{Result, bail};
use serde::Serialize;
use url::Url;

const REQUIRED_SCHEME: &str = "http";

pub struct Client {
    client: reqwest::blocking::Client,
    endpoint: Url,
}

impl Client {
    pub fn new(endpoint: &str) -> Result<Self> {
        let endpoint = Url::parse(endpoint)?;

        if !endpoint.scheme().contains(REQUIRED_SCHEME) {
            bail!("Endpoint must be an http or https url, given: {endpoint}")
        }

        let client = Self {
            client: reqwest::blocking::Client::new(),
            endpoint,
        };
        Ok(client)
    }

    pub fn get_record(&self, args: GetRecordArgs) -> Result<GetRecordResponse> {
        let xml = self.do_query(Query::new(Verb::GetRecord, args))?;
        let response = GetRecordResponse::new(&xml)?;
        Ok(response)
    }

    pub fn identify(&self) -> Result<IdentifyResponse> {
        let xml = self.do_query(Query::new(Verb::Identify, ()))?;
        let response = IdentifyResponse::new(&xml)?;
        Ok(response)
    }

    pub fn list_identifiers(
        &self,
        args: ListIdentifiersArgs,
    ) -> Result<ResumableIter<'_, ListIdentifiersResponse>> {
        ResumableIter::new(self, Verb::ListIdentifiers, args)
    }

    pub fn list_records(
        &self,
        args: ListRecordsArgs,
    ) -> Result<ResumableIter<'_, ListRecordsResponse>> {
        ResumableIter::new(self, Verb::ListRecords, args)
    }

    fn build_url<T: Serialize>(&self, query: Query<T>) -> Result<String> {
        let query = serde_qs::to_string(&query)?;
        let url = format!("{}?{query}", self.endpoint);
        Ok(url)
    }

    pub(crate) fn do_query<T: Serialize>(&self, query: Query<T>) -> Result<String> {
        let url = self.build_url(query)?;
        let user_agent = format!("oai-pmh-rs/{}", env!("CARGO_PKG_VERSION"));
        let xml = self
            .client
            .get(url)
            .header("Accept", "text/xml")
            .header("User-Agent", user_agent)
            .send()?
            .text()?;
        Ok(xml)
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::Verb;
    use crate::client::Client;
    use crate::client::query::{GetRecordArgs, ListIdentifiersArgs, ListRecordsArgs, Query};

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
        let parsed_url = Url::parse(&url).unwrap();

        assert!(parsed_url.host_str() == Some("test.archivesspace.org"));
        assert!(parsed_url.path() == "/oai");
        assert!(
            parsed_url.query()
                == Some(
                    "verb=GetRecord&identifier=oai%3Aarchivesspace%3A%2Frepositories%2F2%2Fresources%2F2&metadataPrefix=oai_ead"
                )
        );
    }

    #[test]
    fn client_build_identify_query_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(Verb::Identify, ());
        let url = client.build_url(query).unwrap();
        let parsed_url = Url::parse(&url).unwrap();

        assert!(parsed_url.host_str() == Some("test.archivesspace.org"));
        assert!(parsed_url.path() == "/oai");
        assert!(parsed_url.query() == Some("verb=Identify"));
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
        let parsed_url = Url::parse(&url).unwrap();

        assert!(parsed_url.host_str() == Some("test.archivesspace.org"));
        assert!(parsed_url.path() == "/oai");
        assert!(
            parsed_url.query() == Some("verb=ListIdentifiers&metadataPrefix=oai_ead&set=speccol")
        );
    }

    #[test]
    fn client_build_list_records_query_url() {
        let endpoint = "https://test.archivesspace.org/oai";
        let client = Client::new(endpoint).unwrap();
        let query = Query::new(Verb::ListRecords, ListRecordsArgs::new("oai_ead"));
        let url = client.build_url(query).unwrap();
        let parsed_url = Url::parse(&url).unwrap();

        assert!(parsed_url.host_str() == Some("test.archivesspace.org"));
        assert!(parsed_url.path() == "/oai");
        assert!(parsed_url.query() == Some("verb=ListRecords&metadataPrefix=oai_ead"));
    }
}
