use crate::client::response::ResponseError;
use quick_xml::DeError;
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
use std::time::Duration;
use thiserror::Error as ThisError;
use url::ParseError;

#[derive(Debug, ThisError)]
pub enum Error {
    /// HTTP request failed (network error, timeout, etc.)
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// The endpoint URL has an invalid scheme (must be http or https)
    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),
    /// Header name is invalid
    #[error("invalid header name: {0}")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    /// Header value is invalid
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    /// Failed to serialize query parameters
    #[error("query serialization failed: {0}")]
    QuerySerialize(#[from] serde_qs::Error),
    /// The server is busy or rate limiting (HTTP 503 or 429)
    #[error("server busy (HTTP {status}){}", retry_after_label(*retry_after))]
    RateLimited {
        /// The HTTP status code (503 or 429)
        status: u16,
        /// The delay requested by the `Retry-After` header, if given in seconds
        retry_after: Option<Duration>,
    },
    /// Response was not valid XML (e.g., HTML error page, plain text)
    #[error("unexpected response{}: {body}", content_type_label(content_type.as_deref()))]
    UnexpectedResponse {
        /// The content-type header, if present
        content_type: Option<String>,
        /// The response body (truncated if too long)
        body: String,
    },
    /// The server returned OAI-PMH errors on a page after the first one in a
    /// resumable stream, so the harvest cannot continue.
    #[error("OAI-PMH error: {}", join_errors(.0))]
    Oai(Vec<ResponseError>),
    /// Internal inconsistency while parsing a response (a bug in this crate)
    #[error("inconsistent response parse: {0}")]
    ParseMismatch(String),
    /// Failed to parse the endpoint URL
    #[error("URL parsing failed: {0}")]
    UrlParse(#[from] ParseError),
    /// Failed to parse XML response
    #[error("XML parsing failed: {0}")]
    XmlParse(#[from] DeError),
}

fn join_errors(errors: &[ResponseError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

fn retry_after_label(retry_after: Option<Duration>) -> String {
    match retry_after {
        Some(delay) => format!(", retry after {}s", delay.as_secs()),
        None => String::new(),
    }
}

fn content_type_label(content_type: Option<&str>) -> String {
    match content_type {
        Some(content_type) => format!(" (content-type: {content_type})"),
        None => String::new(),
    }
}

/// A specialized Result type for oai-pmh operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod display_tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn invalid_endpoint_display() {
        let e = Error::InvalidEndpoint("bad".into());
        assert_eq!(e.to_string(), "invalid endpoint: bad");
        assert!(e.source().is_none());
    }

    #[test]
    fn unexpected_response_with_content_type() {
        let e = Error::UnexpectedResponse {
            content_type: Some("text/html".into()),
            body: "<html>".into(),
        };
        assert_eq!(
            e.to_string(),
            "unexpected response (content-type: text/html): <html>"
        );
        assert!(e.source().is_none());
    }

    #[test]
    fn unexpected_response_without_content_type() {
        let e = Error::UnexpectedResponse {
            content_type: None,
            body: "oops".into(),
        };
        assert_eq!(e.to_string(), "unexpected response: oops");
    }

    #[test]
    fn url_parse_wraps_source() {
        let inner = url::Url::parse("not a url").unwrap_err();
        let e: Error = inner.into();
        assert!(e.to_string().starts_with("URL parsing failed: "));
        assert!(e.source().is_some());
    }
}
