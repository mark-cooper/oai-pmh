use std::fmt;

use quick_xml::DeError;
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue};
use url::ParseError;

#[derive(Debug)]
pub enum Error {
    /// HTTP request failed (network error, timeout, etc.)
    Http(reqwest::Error),

    /// The endpoint URL has an invalid scheme (must be http or https)
    InvalidEndpoint(String),

    /// Header name is invalid
    InvalidHeaderName(InvalidHeaderName),

    /// Header value is invalid
    InvalidHeaderValue(InvalidHeaderValue),

    /// Failed to serialize query parameters
    QuerySerialize(serde_qs::Error),

    /// Response was not valid XML (e.g., HTML error page, plain text)
    UnexpectedResponse {
        /// The content-type header, if present
        content_type: Option<String>,
        /// The response body (truncated if too long)
        body: String,
    },

    /// Failed to parse the endpoint URL
    UrlParse(ParseError),

    /// Failed to parse XML response
    XmlParse(DeError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            Error::InvalidEndpoint(_) => None,
            Error::InvalidHeaderName(e) => Some(e),
            Error::InvalidHeaderValue(e) => Some(e),
            Error::QuerySerialize(e) => Some(e),
            Error::UnexpectedResponse { .. } => None,
            Error::UrlParse(e) => Some(e),
            Error::XmlParse(e) => Some(e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP request failed: {e}"),
            Error::InvalidEndpoint(msg) => write!(f, "invalid endpoint: {msg}"),
            Error::InvalidHeaderName(e) => write!(f, "invalid header name: {e}"),
            Error::InvalidHeaderValue(e) => write!(f, "invalid header value: {e}"),
            Error::QuerySerialize(e) => write!(f, "query serialization failed: {e}"),
            Error::UnexpectedResponse { content_type, body } => match content_type {
                Some(ct) => write!(f, "unexpected response (content-type: {ct}): {body}"),
                None => write!(f, "unexpected response: {body}"),
            },
            Error::UrlParse(e) => write!(f, "URL parsing failed: {e}"),
            Error::XmlParse(e) => write!(f, "XML parsing failed: {e}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<InvalidHeaderName> for Error {
    fn from(err: InvalidHeaderName) -> Self {
        Error::InvalidHeaderName(err)
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(err: InvalidHeaderValue) -> Self {
        Error::InvalidHeaderValue(err)
    }
}

impl From<serde_qs::Error> for Error {
    fn from(err: serde_qs::Error) -> Self {
        Error::QuerySerialize(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::UrlParse(err)
    }
}

impl From<DeError> for Error {
    fn from(err: DeError) -> Self {
        Error::XmlParse(err)
    }
}

/// A specialized Result type for oai-pmh operations.
pub type Result<T> = std::result::Result<T, Error>;
