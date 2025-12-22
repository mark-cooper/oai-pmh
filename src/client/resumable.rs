use crate::error::Result;
use serde::Serialize;

use crate::Verb;
use crate::client::Client;
use crate::client::query::{Query, ResumableArgs};
use crate::client::response::{
    ListIdentifiersResponse, ListRecordsResponse, ListSetsResponse, ResumptionToken,
};

/// Async stream for OAI-PMH verbs that support resumption tokens
pub struct ResumableStream<'a, R> {
    client: &'a Client,
    verb: Verb,
    current_response: Option<R>,
    resumption_token: Option<String>,
}

impl<'a, R> ResumableStream<'a, R>
where
    R: ResumableResponse,
{
    pub(crate) async fn new<Args>(client: &'a Client, verb: Verb, args: Args) -> Result<Self>
    where
        Args: Serialize,
    {
        let xml = client.do_query(Query::new(verb, args)).await?;
        let response = R::from_xml(&xml)?;
        let resumption_token = response.resumption_token();

        Ok(Self {
            client,
            verb,
            current_response: Some(response),
            resumption_token,
        })
    }

    async fn fetch_next(&mut self) -> Result<R> {
        let token = self.resumption_token.take().expect("called without token");

        let xml = self
            .client
            .do_query(Query::new(self.verb, ResumableArgs::new(token)))
            .await?;

        let response = R::from_xml(&xml)?;
        self.resumption_token = response.resumption_token();

        Ok(response)
    }

    /// Returns the next response, or None if out.
    pub async fn next(&mut self) -> Option<Result<R>> {
        // Return buffered response if we have one
        if let Some(response) = self.current_response.take() {
            return Some(Ok(response));
        }

        // Fetch next page if we have a resumption token
        if self.resumption_token.is_some() {
            Some(self.fetch_next().await)
        } else {
            None
        }
    }
}

/// Trait for OAI-PMH responses that support resumption tokens
pub trait ResumableResponse: Sized {
    fn from_xml(xml: &str) -> Result<Self>;
    fn resumption_token(&self) -> Option<String>;

    fn extract_token(token: Option<&ResumptionToken>) -> Option<String> {
        token
            .filter(|t| !t.token.is_empty())
            .map(|t| t.token.clone())
    }
}

macro_rules! resumable {
    ($response:ty) => {
        impl ResumableResponse for $response {
            fn from_xml(xml: &str) -> Result<Self> {
                <$response>::new(xml)
            }

            fn resumption_token(&self) -> Option<String> {
                Self::extract_token(
                    self.payload
                        .as_ref()
                        .and_then(|p| p.resumption_token.as_ref()),
                )
            }
        }
    };
}

resumable!(ListIdentifiersResponse);
resumable!(ListRecordsResponse);
resumable!(ListSetsResponse);
