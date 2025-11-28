use anyhow::Result;
use serde::Serialize;

use crate::Verb;
use crate::client::Client;
use crate::client::query::{Query, ResumableArgs};
use crate::client::response::{
    Header, ListIdentifiersResponse, ListRecordsResponse, Record, ResumptionToken,
};

/// Trait for OAI-PMH responses that support resumption tokens
pub trait ResumableResponse: Sized {
    type Item;

    fn from_xml(xml: &str) -> Result<Self>;
    fn into_parts(self) -> (Vec<Self::Item>, Option<String>);
    fn try_token(token: Option<ResumptionToken>) -> Option<String> {
        token.and_then(|t| {
            if !t.token.is_empty() {
                Some(t.token)
            } else {
                None
            }
        })
    }
}

/// Iterator for OAI-PMH verbs that support resumption tokens
pub struct ResumableIter<'a, T, R> {
    client: &'a Client,
    verb: Verb,
    items: std::vec::IntoIter<T>,
    resumption_token: Option<String>,
    _phantom: std::marker::PhantomData<R>,
}

impl<'a, T, R> ResumableIter<'a, T, R>
where
    R: ResumableResponse<Item = T>,
{
    pub(crate) fn new<Args>(client: &'a Client, verb: Verb, args: Args) -> Result<Self>
    where
        Args: Serialize,
    {
        let xml = client.do_query(Query::new(verb, args))?;
        let response = R::from_xml(&xml)?;

        let (items_vec, resumption_token) = response.into_parts();
        let items = items_vec.into_iter();

        Ok(Self {
            client,
            verb,
            items,
            resumption_token,
            _phantom: std::marker::PhantomData,
        })
    }

    fn fetch_next(&mut self) -> Result<()> {
        let token = self
            .resumption_token
            .take()
            .ok_or_else(|| anyhow::anyhow!("No resumption token available"))?;

        let xml = self
            .client
            .do_query(Query::new(self.verb, ResumableArgs::new(token)))?;

        let response = R::from_xml(&xml)?;
        let (items_vec, resumption_token) = response.into_parts();
        self.items = items_vec.into_iter();
        self.resumption_token = resumption_token;

        Ok(())
    }
}

impl<'a, T, R> Iterator for ResumableIter<'a, T, R>
where
    R: ResumableResponse<Item = T>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Try to get next item from current batch
        if let Some(item) = self.items.next() {
            return Some(Ok(item));
        }

        // No more items in current batch, try to fetch next batch
        if self.resumption_token.is_some() {
            match self.fetch_next() {
                Ok(()) => self.items.next().map(Ok),
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }
}

// Trait implementation for ListIdentifiersResponse
impl ResumableResponse for ListIdentifiersResponse {
    type Item = Header;

    fn from_xml(xml: &str) -> Result<Self> {
        ListIdentifiersResponse::new(xml)
    }

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        match self.payload {
            Some(payload) => {
                let items = payload.header;
                (items, Self::try_token(payload.resumption_token))
            }
            None => (Vec::new(), None),
        }
    }
}

// Trait implementation for ListRecordsResponse
impl ResumableResponse for ListRecordsResponse {
    type Item = Record;

    fn from_xml(xml: &str) -> Result<Self> {
        ListRecordsResponse::new(xml)
    }

    fn into_parts(self) -> (Vec<Self::Item>, Option<String>) {
        match self.payload {
            Some(payload) => {
                let items = payload.record;
                (items, Self::try_token(payload.resumption_token))
            }
            None => (Vec::new(), None),
        }
    }
}
