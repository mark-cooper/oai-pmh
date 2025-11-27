use anyhow::Result;
use serde::Deserialize;
use std::fmt;

use crate::client::metadata;

// Response error implementation
#[derive(Debug, Deserialize)]
pub struct ResponseError {
    #[serde(rename = "@code")]
    pub code: ErrorCode,

    #[serde(rename = "$value")]
    pub message: String,
}

impl std::error::Error for ResponseError {}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

// Response error codes
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

// Generate common response structure
// (preferring this to generics ['cus deserialization issues] and no direct struct embed)
macro_rules! response {
    ($name:ident, $payload_name:literal, $payload_type:ty) => {
        #[derive(Debug, Deserialize)]
        #[serde(rename = "OAI-PMH")]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub response_date: String,
            pub request: String,

            #[serde(default)]
            pub error: Option<ResponseError>,

            #[serde(rename = $payload_name, default)]
            pub payload: Option<$payload_type>,
        }

        impl $name {
            pub fn is_err(&self) -> bool {
                self.error.is_some()
            }
        }
    };
}

response!(GetRecordResponse, "GetRecord", GetRecord);
impl GetRecordResponse {
    pub fn new(xml: String) -> Result<Self> {
        let mut response: Self = quick_xml::de::from_str(xml.as_str())?;

        let metadata = metadata::extract_metadata(xml.as_str())
            .into_iter()
            .next()
            .unwrap_or_default();

        if let Some(ref mut payload) = response.payload {
            payload.record.metadata = metadata;
        }

        Ok(response)
    }
}

#[derive(Debug, Deserialize)]
pub struct GetRecord {
    #[serde(rename = "record")]
    pub record: Record,
}

// Identify implementation
response!(IdentifyResponse, "Identify", Identify);
impl IdentifyResponse {
    pub fn new(xml: String) -> Result<Self> {
        let response: Self = quick_xml::de::from_str(xml.as_str())?;
        Ok(response)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identify {
    pub repository_name: String,
    #[serde(rename = "baseURL")]
    pub base_url: String,
    pub protocol_version: String,

    #[serde(default)]
    pub admin_email: Vec<String>,
    pub earliest_datestamp: String,
    pub deleted_record: String,
    pub granularity: String,

    #[serde(default)]
    pub compression: Vec<String>,

    #[serde(skip)]
    pub description: Vec<String>,
}

response!(ListRecordsResponse, "ListRecords", ListRecords);
impl ListRecordsResponse {
    pub fn new(xml: String) -> Result<Self> {
        let mut response: Self = quick_xml::de::from_str(xml.as_str())?;

        let metadata = metadata::extract_metadata(xml.as_str());

        if let Some(ref mut payload) = response.payload {
            for (record, meta) in payload.record.iter_mut().zip(metadata) {
                record.metadata = meta;
            }
        }

        Ok(response)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRecords {
    #[serde(rename = "record")]
    pub record: Vec<Record>,

    #[serde(default)]
    pub resumption_token: Option<ResumptionToken>,
}

// General elements
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub identifier: String,
    pub datestamp: String,

    #[serde(rename = "@status", default)]
    pub status: Option<String>,

    #[serde(default)]
    pub set_spec: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub header: Header,

    #[serde(skip)]
    pub metadata: String,

    #[serde(default)]
    pub about: String,
}

#[derive(Debug, Deserialize)]
pub struct ResumptionToken {
    #[serde(rename = "$value")]
    pub token: String,

    #[serde(rename = "@expirationDate", default)]
    pub expiration_date: Option<String>,

    #[serde(rename = "@completeListSize", default)]
    pub complete_list_size: Option<u64>,

    #[serde(rename = "@cursor", default)]
    pub cursor: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_err_cannot_disseminate_format() {
        let xml = std::fs::read_to_string("tests/fixtures/err_bad_prefix.xml")
            .expect("Failed to load fixture");

        let response = GetRecordResponse::new(xml).unwrap();
        assert!(response.is_err());

        let error = response.error.unwrap();
        assert_eq!(error.code, ErrorCode::CannotDisseminateFormat);
        assert_eq!(
            error.message,
            "The metadata format identified by the value given for the metadataPrefix argument is not supported by the item or by the repository."
        );
    }

    #[test]
    fn test_err_id_does_not_exist() {
        let xml = std::fs::read_to_string("tests/fixtures/err_not_found.xml")
            .expect("Failed to load fixture");

        let response = GetRecordResponse::new(xml).unwrap();
        assert!(response.is_err());

        let error = response.error.unwrap();
        assert_eq!(error.code, ErrorCode::IdDoesNotExist);
        assert_eq!(
            error.message,
            "The value of the identifier argument is unknown or illegal in this repository."
        );
    }

    #[test]
    fn test_get_record_success() {
        let xml = std::fs::read_to_string("tests/fixtures/get_record.xml")
            .expect("Failed to load fixture");

        let response = GetRecordResponse::new(xml).unwrap();
        assert!(!response.is_err());
        assert_eq!(response.response_date, "2025-11-26T19:16:06Z");
        assert_eq!(response.request, "https://test.archivesspace.org");

        let payload = response.payload.unwrap();
        assert_eq!(
            payload.record.header.identifier,
            "oai:archivesspace:/repositories/2/resources/2"
        );
        assert_eq!(payload.record.header.datestamp, "2025-11-11T14:28:08Z");

        let metadata = payload.record.metadata;
        assert!(metadata.contains("<ead"));
        assert!(metadata.contains("xmlns=\"urn:isbn:1-931666-22-9\""));
        assert!(metadata.contains("</ead>"));
    }

    #[test]
    fn test_identify_success() {
        let xml =
            std::fs::read_to_string("tests/fixtures/identify.xml").expect("Failed to load fixture");

        let response = IdentifyResponse::new(xml).unwrap();
        assert!(!response.is_err());
        assert_eq!(response.response_date, "2025-11-26T21:49:54Z");
        assert_eq!(response.request, "https://test.archivesspace.org");

        let payload = response.payload.unwrap();
        assert_eq!(payload.repository_name, "ArchivesSpace OAI Provider");
        assert_eq!(payload.base_url, "https://test.archivesspace.org");
        assert_eq!(payload.protocol_version, "2.0");
        assert_eq!(payload.admin_email, vec!["admin@example.com"]);
        assert_eq!(payload.earliest_datestamp, "1970-01-01T00:00:00Z");
        assert_eq!(payload.deleted_record, "persistent");
        assert_eq!(payload.granularity, "YYYY-MM-DDThh:mm:ssZ");
        assert!(payload.compression.is_empty());
    }

    #[test]
    fn test_list_records_success() {
        let xml = std::fs::read_to_string("tests/fixtures/list_records.xml")
            .expect("Failed to load fixture");

        let response = ListRecordsResponse::new(xml).unwrap();
        assert!(!response.is_err());
        assert_eq!(response.response_date, "2025-11-27T02:10:07Z");
        assert_eq!(response.request, "https://test.archivesspace.org");

        let payload = response.payload.unwrap();

        let token = payload.resumption_token.as_ref().unwrap();
        assert_eq!(
            token.token,
            "eyJtZXRhZGF0YV9wcmVmaXgiOiJvYWlfZGMiLCJmcm9tIjoiMTk3MC0wMS0wMSAwMDowMDowMCBVVEMiLCJ1bnRpbCI6IjIwMjUtMTEtMjcgMDI6MTA6MDYgVVRDIiwic3RhdGUiOiJwcm9kdWNpbmdfcmVjb3JkcyIsImxhc3RfZGVsZXRlX2lkIjowLCJyZW1haW5pbmdfdHlwZXMiOnsiUmVzb3VyY2UiOjAsIkFyY2hpdmFsT2JqZWN0IjoyNX0sImlzc3VlX3RpbWUiOjE3NjQyMDk0MDc3MzN9"
        );

        assert!(payload.record.len() > 1);

        // Verify each record has correct metadata pairing
        let test_cases = [
            (
                "oai:archivesspace:/repositories/2/archival_objects/1",
                "Correspondence about art, 1974–2014",
            ),
            (
                "oai:archivesspace:/repositories/2/archival_objects/2",
                "Correspondence about Veterans Affairs appeals, 1945–2012",
            ),
            (
                "oai:archivesspace:/repositories/2/archival_objects/3",
                "Correspondence relating to family, 1922–1972, undated",
            ),
        ];

        for (idx, (expected_id, expected_title)) in test_cases.iter().enumerate() {
            let record = &payload.record[idx];
            assert_eq!(record.header.identifier, *expected_id);
            assert_eq!(record.header.datestamp, "2025-11-11T00:31:42Z");
            assert!(record.metadata.contains("<oai_dc:dc"));
            assert!(record.metadata.contains(expected_title));
        }

        // Ensure all records have non-empty metadata
        for record in &payload.record {
            assert!(!record.metadata.is_empty());
            assert!(record.metadata.contains("<oai_dc:dc"));
        }
    }
}
