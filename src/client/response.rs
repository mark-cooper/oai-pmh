use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(rename = "OAI-PMH")]
pub struct OaiResponse {
    #[serde(rename = "responseDate")]
    pub response_date: String,
    #[serde(rename = "request")]
    pub request: String,
    #[serde(rename = "error", default)]
    pub error: Option<OaiError>,
}

impl OaiResponse {
    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }
}

#[derive(Debug, Deserialize)]
pub struct OaiError {
    #[serde(rename = "@code")]
    pub code: ErrorCode,
    #[serde(rename = "$value")]
    pub message: String,
}

impl std::error::Error for OaiError {}

impl fmt::Display for OaiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    #[serde(rename = "badArgument")]
    BadArgument,
    #[serde(rename = "badResumptionToken")]
    BadResumptionToken,
    #[serde(rename = "badVerb")]
    BadVerb,
    #[serde(rename = "cannotDisseminateFormat")]
    CannotDisseminateFormat,
    #[serde(rename = "idDoesNotExist")]
    IdDoesNotExist,
    #[serde(rename = "noRecordsMatch")]
    NoRecordsMatch,
    #[serde(rename = "noMetadataFormats")]
    NoMetadataFormats,
    #[serde(rename = "noSetHierarchy")]
    NoSetHierarchy,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::BadArgument => write!(f, "badArgument"),
            ErrorCode::BadResumptionToken => write!(f, "badResumptionToken"),
            ErrorCode::BadVerb => write!(f, "badVerb"),
            ErrorCode::CannotDisseminateFormat => write!(f, "cannotDisseminateFormat"),
            ErrorCode::IdDoesNotExist => write!(f, "idDoesNotExist"),
            ErrorCode::NoRecordsMatch => write!(f, "noRecordsMatch"),
            ErrorCode::NoMetadataFormats => write!(f, "noMetadataFormats"),
            ErrorCode::NoSetHierarchy => write!(f, "noSetHierarchy"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_cannot_disseminate_format() {
        let xml = std::fs::read_to_string("tests/fixtures/err_bad_prefix.xml")
            .expect("Failed to load fixture");

        let response: OaiResponse = quick_xml::de::from_str(xml.as_str()).unwrap();
        assert!(response.is_err());

        let error = response.error.unwrap();
        assert_eq!(error.code, ErrorCode::CannotDisseminateFormat);
        assert_eq!(
            error.message,
            "The metadata format identified by the value given for the metadataPrefix argument is not supported by the item or by the repository."
        );
    }

    #[test]
    fn test_deserialize_id_does_not_exist() {
        let xml = std::fs::read_to_string("tests/fixtures/err_not_found.xml")
            .expect("Failed to load fixture");

        let response: OaiResponse = quick_xml::de::from_str(xml.as_str()).unwrap();
        assert!(response.is_err());

        let error = response.error.unwrap();
        assert_eq!(error.code, ErrorCode::IdDoesNotExist);
        assert_eq!(
            error.message,
            "The value of the identifier argument is unknown or illegal in this repository."
        );
    }
}
