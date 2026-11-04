use crate::client::metadata;
use crate::error::{Error, Result};
use quick_xml::de;
use serde::Deserialize;
use std::fmt;
use thiserror::Error as ThisError;

// Response error implementation
#[derive(Debug, Deserialize, ThisError)]
#[error("{code}: {message}")]
pub struct ResponseError {
    #[serde(rename = "@code")]
    pub code: ErrorCode,
    // The spec makes the message optional; servers often send e.g. `<error code="noRecordsMatch"/>`.
    #[serde(rename = "$value", default)]
    pub message: String,
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
        let s = match self {
            ErrorCode::BadArgument => "badArgument",
            ErrorCode::BadResumptionToken => "badResumptionToken",
            ErrorCode::BadVerb => "badVerb",
            ErrorCode::CannotDisseminateFormat => "cannotDisseminateFormat",
            ErrorCode::IdDoesNotExist => "idDoesNotExist",
            ErrorCode::NoRecordsMatch => "noRecordsMatch",
            ErrorCode::NoMetadataFormats => "noMetadataFormats",
            ErrorCode::NoSetHierarchy => "noSetHierarchy",
        };
        f.write_str(s)
    }
}

/// Parse an OAI-PMH response from XML, with a hook for data serde can't capture
/// (e.g. raw `<metadata>` XML).
pub trait FromXml: Sized + serde::de::DeserializeOwned {
    fn post_process(&mut self, _xml: &str) -> Result<()> {
        Ok(())
    }

    fn from_xml(xml: &str) -> Result<Self> {
        let mut response: Self = de::from_str(xml)?;
        response.post_process(xml)?;
        Ok(response)
    }
}

// Generate common response structure
// (preferring this to generics ['cus deserialization issues] and no direct struct embed)
macro_rules! response {
    ($name:ident, $payload_name:literal, $payload_type:ty) => {
        response!($name, $payload_name, $payload_type; custom);
        impl FromXml for $name {}
    };
    ($name:ident, $payload_name:literal, $payload_type:ty; custom) => {
        #[derive(Debug, Deserialize)]
        #[serde(rename = "OAI-PMH")]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub response_date: String,
            pub request: String,

            /// A response may carry several `<error>` elements.
            #[serde(default)]
            pub error: Vec<ResponseError>,

            #[serde(rename = $payload_name, default)]
            pub payload: Option<$payload_type>,
        }

        impl $name {
            pub fn new(xml: &str) -> Result<Self> {
                <Self as FromXml>::from_xml(xml)
            }

            pub fn is_err(&self) -> bool {
                !self.error.is_empty()
            }
        }
    };
}

response!(GetRecordResponse, "GetRecord", GetRecord; custom);
impl FromXml for GetRecordResponse {
    fn post_process(&mut self, xml: &str) -> Result<()> {
        let metadata = metadata::extract_record_metadata(xml)?
            .into_iter()
            .next()
            .flatten()
            .unwrap_or_default();

        if let Some(payload) = self.payload.as_mut() {
            payload.record.metadata = metadata;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct GetRecord {
    #[serde(rename = "record")]
    pub record: Record,
}

// Identify implementation
response!(IdentifyResponse, "Identify", Identify);

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

// ListIdentifiers implementation
response!(ListIdentifiersResponse, "ListIdentifiers", ListIdentifiers);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIdentifiers {
    #[serde(rename = "header")]
    pub header: Vec<Header>,

    #[serde(default)]
    pub resumption_token: Option<ResumptionToken>,
}

// ListMetadataFormats implementation
response!(
    ListMetadataFormatsResponse,
    "ListMetadataFormats",
    ListMetadataFormats
);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMetadataFormats {
    #[serde(rename = "metadataFormat")]
    pub metadata_format: Vec<MetadataFormat>,
}

// ListRecords implementation
response!(ListRecordsResponse, "ListRecords", ListRecords; custom);
impl FromXml for ListRecordsResponse {
    fn post_process(&mut self, xml: &str) -> Result<()> {
        let metadata = metadata::extract_record_metadata(xml)?;

        if let Some(payload) = self.payload.as_mut() {
            // Metadata is matched to records by position, so the two parsers must agree.
            if metadata.len() != payload.record.len() {
                return Err(Error::ParseMismatch(format!(
                    "found {} records but {} metadata slots",
                    payload.record.len(),
                    metadata.len()
                )));
            }
            for (record, meta) in payload.record.iter_mut().zip(metadata) {
                record.metadata = meta.unwrap_or_default();
            }
        }
        Ok(())
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

// ListSets implementation
response!(ListSetsResponse, "ListSets", ListSets);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSets {
    pub set: Vec<Set>,

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
pub struct MetadataFormat {
    pub metadata_prefix: String,
    pub metadata_namespace: String,
    pub schema: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub header: Header,

    #[serde(skip)]
    pub metadata: String,

    /// Not yet populated. `<about>` holds arbitrary XML (and may repeat), which serde
    /// can't read into a `String`, so it is skipped to avoid failing the whole response.
    #[serde(skip)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Set {
    pub set_spec: String,
    pub set_name: String,

    #[serde(skip)]
    pub set_description: Vec<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_nested_record_elements_do_not_misalign_metadata() {
        let xml = r#"<OAI-PMH><responseDate>x</responseDate><request>y</request><ListRecords>
<record><header><identifier>1</identifier><datestamp>d</datestamp></header>
<metadata><m>1</m></metadata>
<about><record><metadata><z/></metadata></record></about></record>
<record><header><identifier>2</identifier><datestamp>d</datestamp></header>
<metadata><record><metadata><z/></metadata></record></metadata></record>
</ListRecords></OAI-PMH>"#;

        let records = ListRecordsResponse::new(xml).unwrap().payload.unwrap().record;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].metadata, "<m>1</m>");
        assert!(records[1].metadata.starts_with("<record>"));
    }

    fn identify_xml(errors: &str) -> String {
        format!(
            "<OAI-PMH><responseDate>x</responseDate><request>y</request>{errors}</OAI-PMH>"
        )
    }

    #[test]
    fn test_multiple_errors() {
        let xml = identify_xml(
            r#"<error code="badArgument">a</error><error code="badResumptionToken">b</error>"#,
        );
        let response = IdentifyResponse::new(&xml).unwrap();
        assert!(response.is_err());
        assert_eq!(response.error.len(), 2);
        assert_eq!(response.error[0].code, ErrorCode::BadArgument);
        assert_eq!(response.error[1].code, ErrorCode::BadResumptionToken);
    }

    #[test]
    fn test_error_without_message() {
        let xml = identify_xml(r#"<error code="noRecordsMatch"/>"#);
        let response = IdentifyResponse::new(&xml).unwrap();
        assert_eq!(response.error.len(), 1);
        assert_eq!(response.error[0].code, ErrorCode::NoRecordsMatch);
        assert!(response.error[0].message.is_empty());
    }

    #[test]
    fn test_about_with_nested_xml_does_not_fail_parsing() {
        let xml = r#"<OAI-PMH><responseDate>x</responseDate><request>y</request>
<GetRecord><record>
<header><identifier>i</identifier><datestamp>d</datestamp></header>
<metadata><a/></metadata>
<about><r:rights xmlns:r="u"><r:a>1</r:a></r:rights></about>
<about><b>2</b></about>
</record></GetRecord></OAI-PMH>"#;

        let response = GetRecordResponse::new(xml).expect("about should not break parsing");
        let record = response.payload.unwrap().record;
        assert_eq!(record.header.identifier, "i");
        assert!(record.about.is_empty());
    }

    use super::*;
    use std::fs;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::BadArgument.to_string(), "badArgument");
        assert_eq!(
            ErrorCode::BadResumptionToken.to_string(),
            "badResumptionToken"
        );
        assert_eq!(ErrorCode::BadVerb.to_string(), "badVerb");
        assert_eq!(
            ErrorCode::CannotDisseminateFormat.to_string(),
            "cannotDisseminateFormat"
        );
        assert_eq!(ErrorCode::IdDoesNotExist.to_string(), "idDoesNotExist");
        assert_eq!(ErrorCode::NoRecordsMatch.to_string(), "noRecordsMatch");
        assert_eq!(
            ErrorCode::NoMetadataFormats.to_string(),
            "noMetadataFormats"
        );
        assert_eq!(ErrorCode::NoSetHierarchy.to_string(), "noSetHierarchy");
    }

    #[test]
    fn test_response_error_display() {
        let e = ResponseError {
            code: ErrorCode::IdDoesNotExist,
            message: "unknown id".into(),
        };
        assert_eq!(e.to_string(), "idDoesNotExist: unknown id");
    }

    #[test]
    fn test_err_cannot_disseminate_format() {
        let xml = fs::read_to_string("tests/fixtures/err_bad_prefix.xml")
            .expect("Failed to load fixture");

        let response = GetRecordResponse::new(&xml).unwrap();
        assert!(response.is_err());

        assert_eq!(response.error.len(), 1);
        let error = &response.error[0];
        assert_eq!(error.code, ErrorCode::CannotDisseminateFormat);
        assert_eq!(
            error.message,
            "The metadata format identified by the value given for the metadataPrefix argument is not supported by the item or by the repository."
        );
    }

    #[test]
    fn test_err_id_does_not_exist() {
        let xml =
            fs::read_to_string("tests/fixtures/err_not_found.xml").expect("Failed to load fixture");

        let response = GetRecordResponse::new(&xml).unwrap();
        assert!(response.is_err());

        assert_eq!(response.error.len(), 1);
        let error = &response.error[0];
        assert_eq!(error.code, ErrorCode::IdDoesNotExist);
        assert_eq!(
            error.message,
            "The value of the identifier argument is unknown or illegal in this repository."
        );
    }

    #[test]
    fn test_get_record_success() {
        let xml =
            fs::read_to_string("tests/fixtures/get_record.xml").expect("Failed to load fixture");

        let response = GetRecordResponse::new(&xml).unwrap();
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
            fs::read_to_string("tests/fixtures/identify.xml").expect("Failed to load fixture");

        let response = IdentifyResponse::new(&xml).unwrap();
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
    fn test_list_identifiers_success() {
        let xml = fs::read_to_string("tests/fixtures/list_identifiers.xml")
            .expect("Failed to load fixture");

        let response = ListIdentifiersResponse::new(&xml).unwrap();
        assert!(!response.is_err());
        assert_eq!(response.response_date, "2025-11-26T21:58:16Z");
        assert_eq!(response.request, "https://test.archivesspace.org");

        let payload = response.payload.unwrap();

        let token = payload.resumption_token.as_ref().unwrap();
        assert_eq!(
            token.token,
            "eyJtZXRhZGF0YV9wcmVmaXgiOiJvYWlfZWFkIiwiZnJvbSI6IjE5NzAtMDEtMDEgMDA6MDA6MDAgVVRDIiwidW50aWwiOiIyMDI1LTExLTI2IDIxOjU4OjE1IFVUQyIsInN0YXRlIjoicHJvZHVjaW5nX3JlY29yZHMiLCJsYXN0X2RlbGV0ZV9pZCI6MCwicmVtYWluaW5nX3R5cGVzIjp7IlJlc291cmNlIjoyfSwiaXNzdWVfdGltZSI6MTc2NDE5NDI5NjA2Mn0="
        );

        assert!(!payload.header.is_empty());

        let test_cases = [(
            "oai:archivesspace:/repositories/2/resources/2",
            "2025-11-11T14:28:08Z",
        )];

        for (idx, (expected_id, expected_datestamp)) in test_cases.iter().enumerate() {
            let header = &payload.header[idx];
            assert_eq!(header.identifier, *expected_id);
            assert_eq!(header.datestamp, *expected_datestamp);
        }
    }

    #[test]
    fn test_list_metadata_formats_success() {
        let xml = fs::read_to_string("tests/fixtures/list_metadata_formats.xml")
            .expect("Failed to load fixture");

        let response = ListMetadataFormatsResponse::new(&xml).unwrap();
        assert!(!response.is_err());
        assert_eq!(response.response_date, "2025-11-27T05:39:57Z");
        assert_eq!(response.request, "https://test.archivesspace.org");

        let payload = response.payload.unwrap();
        if let Some(format) = payload.metadata_format.into_iter().next() {
            assert_eq!(format.metadata_prefix, "oai_dc");
            assert_eq!(
                format.metadata_namespace,
                "http://www.openarchives.org/OAI/2.0/oai_dc/"
            );
            assert_eq!(
                format.schema,
                "http://www.openarchives.org/OAI/2.0/oai_dc.xsd"
            );
        }
    }

    #[test]
    fn test_list_records_success() {
        let xml =
            fs::read_to_string("tests/fixtures/list_records.xml").expect("Failed to load fixture");

        let response = ListRecordsResponse::new(&xml).unwrap();
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

    #[test]
    fn test_list_records_deleted_records_have_empty_metadata() {
        let xml = fs::read_to_string("tests/fixtures/list_records_with_deleted.xml")
            .expect("Failed to load fixture");

        let response = ListRecordsResponse::new(&xml).unwrap();
        assert!(!response.is_err());

        let payload = response.payload.unwrap();
        assert_eq!(payload.record.len(), 2);

        let deleted = &payload.record[0];
        assert_eq!(deleted.header.status.as_deref(), Some("deleted"));
        assert!(deleted.metadata.is_empty());

        let alive = &payload.record[1];
        assert!(alive.metadata.contains("<oai_dc:dc"));
        assert!(alive.metadata.contains("<dc:title>Alive</dc:title>"));
    }

    #[test]
    fn test_list_sets_success() {
        let xml =
            fs::read_to_string("tests/fixtures/list_sets.xml").expect("Failed to load fixture");

        let response = ListSetsResponse::new(&xml).unwrap();
        assert!(!response.is_err());
        assert_eq!(response.response_date, "2025-11-27T05:38:04Z");
        assert_eq!(response.request, "https://test.archivesspace.org");

        let payload = response.payload.unwrap();

        assert!(!payload.set.is_empty());

        let test_cases = ["class", "collection", "file"];

        for (idx, expected) in test_cases.iter().enumerate() {
            let set = &payload.set[idx];
            assert_eq!(set.set_spec, *expected);
            assert_eq!(set.set_name, *expected);
        }
    }
}
