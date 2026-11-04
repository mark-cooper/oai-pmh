use crate::error::{Error, Result};
use serde::Serialize;

use crate::Verb;
use crate::client::Client;
use crate::client::query::{Query, ResumableArgs};
use crate::client::response::{
    FromXml, ListIdentifiersResponse, ListRecordsResponse, ListSetsResponse, ResponseError,
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
        let token = self.resumption_token.clone().expect("called without token");

        let xml = self
            .client
            .do_query(Query::new(self.verb, ResumableArgs::new(token)))
            .await?;

        let response = R::from_xml(&xml)?;

        // An error on a continuation page means the harvest is incomplete. Without this the
        // stream would just end (no payload, so no token) and look like a normal finish.
        if !response.errors().is_empty() {
            return Err(Error::Oai(response.into_errors()));
        }

        // Only advance once the page has fetched and parsed successfully.
        self.resumption_token = response.resumption_token();

        Ok(response)
    }

    /// Returns the next response, or `Ok(None)` if the stream is exhausted.
    pub async fn try_next(&mut self) -> Result<Option<R>> {
        // Return buffered response if we have one
        if let Some(response) = self.current_response.take() {
            return Ok(Some(response));
        }

        // Fetch next page if we have a resumption token
        if self.resumption_token.is_some() {
            Ok(Some(self.fetch_next().await?))
        } else {
            Ok(None)
        }
    }

    /// Returns the next response, or `None` if out.
    pub async fn next(&mut self) -> Option<Result<R>> {
        self.try_next().await.transpose()
    }
}

/// Trait for OAI-PMH responses that support resumption tokens
pub trait ResumableResponse: FromXml {
    fn resumption_token(&self) -> Option<String>;
    fn errors(&self) -> &[ResponseError];
    fn into_errors(self) -> Vec<ResponseError>;
}

macro_rules! resumable {
    ($response:ty) => {
        impl ResumableResponse for $response {
            fn errors(&self) -> &[ResponseError] {
                &self.error
            }

            fn into_errors(self) -> Vec<ResponseError> {
                self.error
            }

            fn resumption_token(&self) -> Option<String> {
                self.payload
                    .as_ref()
                    .and_then(|p| p.resumption_token.as_ref())
                    .filter(|t| !t.token.is_empty())
                    .map(|t| t.token.clone())
            }
        }
    };
}

resumable!(ListIdentifiersResponse);
resumable!(ListRecordsResponse);
resumable!(ListSetsResponse);
