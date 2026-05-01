use quick_xml::DeError;
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
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
    /// Response was not valid XML (e.g., HTML error page, plain text)
    #[error("unexpected response{}: {body}", content_type_label(content_type.as_deref()))]
    UnexpectedResponse {
        /// The content-type header, if present
        content_type: Option<String>,
        /// The response body (truncated if too long)
        body: String,
    },
    /// Failed to parse the endpoint URL
    #[error("URL parsing failed: {0}")]
    UrlParse(#[from] ParseError),
    /// Failed to parse XML response
    #[error("XML parsing failed: {0}")]
    XmlParse(#[from] DeError),
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
