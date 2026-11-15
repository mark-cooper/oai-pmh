use crate::error::{Error, Result};
use serde::Serialize;
use std::marker::PhantomData;

use crate::Verb;
use crate::client::Client;
use crate::client::query::{Query, ResumableArgs};
use crate::client::response::{
    ErrorCode, FromXml, ListIdentifiersResponse, ListRecordsResponse, ListSetsResponse,
    ResponseError,
};

/// Async stream for OAI-PMH verbs that support resumption tokens.
///
/// No request is made until the first call to [`try_next`](Self::try_next).
/// Responses carrying OAI-PMH errors are returned as `Err(Error::Oai(..))`, except
/// `noRecordsMatch` on the first page, which ends the stream (an empty list).
pub struct ResumableStream<'a, R> {
    client: &'a Client,
    verb: Verb,
    next: Next,
    _response: PhantomData<R>,
}

/// The request the next `try_next()` will make.
enum Next {
    /// The initial request, as a serialized query string.
    Start(String),
    /// A continuation request for this resumption token.
    Resume(String),
    /// The query args could not be serialized; reported by the next `try_next()`.
    Invalid(Error),
    /// The stream is exhausted.
    Done,
}

impl<'a, R> ResumableStream<'a, R>
where
    R: ResumableResponse,
{
    pub(crate) fn new<Args>(client: &'a Client, verb: Verb, args: Args) -> Self
    where
        Args: Serialize,
    {
        let next = match serde_qs::to_string(&Query::new(verb, args)) {
            Ok(query) => Next::Start(query),
            Err(err) => Next::Invalid(err.into()),
        };

        Self {
            client,
            verb,
            next,
            _response: PhantomData,
        }
    }

    /// Returns the next response, or `Ok(None)` if the stream is exhausted.
    ///
    /// A failed request leaves the stream where it was, so calling this again
    /// retries the same page (the first one included).
    pub async fn try_next(&mut self) -> Result<Option<R>> {
        let query = match &self.next {
            Next::Start(query) => query.clone(),
            Next::Resume(token) => {
                serde_qs::to_string(&Query::new(self.verb, ResumableArgs::new(token.as_str())))?
            }
            Next::Invalid(_) => match std::mem::replace(&mut self.next, Next::Done) {
                Next::Invalid(err) => return Err(err),
                _ => unreachable!(),
            },
            Next::Done => return Ok(None),
        };

        let xml = self.client.do_raw_query(&query).await?;
        let response = R::from_xml(&xml)?;

        if !response.errors().is_empty() {
            // noRecordsMatch on the first request just means the list is empty.
            let is_first = matches!(self.next, Next::Start(_));
            let no_records = response
                .errors()
                .iter()
                .all(|e| e.code == ErrorCode::NoRecordsMatch);
            if is_first && no_records {
                self.next = Next::Done;
                return Ok(None);
            }

            // Anything else is a failed request. On a continuation page this also stops
            // an incomplete harvest from looking like a normal finish.
            return Err(Error::Oai(response.into_errors()));
        }

        // Only advance once the page has fetched and parsed successfully.
        self.next = response.resumption_token().map_or(Next::Done, Next::Resume);

        Ok(Some(response))
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
