use crate::error::{Error, Result};
use serde::Serialize;

use crate::Verb;
use crate::client::Client;
use crate::client::query::{Query, ResumableArgs};
use crate::client::response::{
    ListIdentifiersResponse, ListRecordsResponse, ListSetsResponse, ResumptionToken,
};

/// Iterator for OAI-PMH verbs that support resumption tokens
pub struct ResumableIter<'a, R> {
    client: &'a Client,
    verb: Verb,
    current_response: Option<R>,
    resumption_token: Option<String>,
}

impl<'a, R> ResumableIter<'a, R>
where
    R: ResumableResponse,
{
    pub(crate) fn new<Args>(client: &'a Client, verb: Verb, args: Args) -> Result<Self>
    where
        Args: Serialize,
    {
        let xml = client.do_query(Query::new(verb, args))?;
        let response = R::from_xml(&xml)?;
        let resumption_token = response.resumption_token();

        Ok(Self {
            client,
            verb,
            current_response: Some(response),
            resumption_token,
        })
    }

    fn fetch_next(&mut self) -> Result<()> {
        let token = self
            .resumption_token
            .take()
            .ok_or(Error::NoResumptionToken)?;

        let xml = self
            .client
            .do_query(Query::new(self.verb, ResumableArgs::new(token)))?;

        let response = R::from_xml(&xml)?;
        self.resumption_token = response.resumption_token();
        self.current_response = Some(response);

        Ok(())
    }
}

impl<'a, R> Iterator for ResumableIter<'a, R>
where
    R: ResumableResponse,
{
    type Item = Result<R>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return current response if we have one
        if let Some(response) = self.current_response.take() {
            return Some(Ok(response));
        }

        // Try to fetch next batch if we have a resumption token
        if self.resumption_token.is_some() {
            Some(match self.fetch_next() {
                Ok(()) => Ok(self.current_response.take()?),
                Err(e) => Err(e),
            })
        } else {
            None
        }
    }
}

/// Trait for OAI-PMH responses that support resumption tokens
pub trait ResumableResponse: Sized {
    fn from_xml(xml: &str) -> Result<Self>;
    fn resumption_token(&self) -> Option<String>;

    /// Helper to extract non-empty token string from ResumptionToken
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
