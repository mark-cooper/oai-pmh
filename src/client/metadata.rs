use once_cell::sync::Lazy;
use regex::Regex;

// Match content between <metadata> and </metadata> tags
// Using non-greedy match (.*?) to handle multiple metadata elements
// (?s) flag makes . match newlines (dotall mode)
static METADATA_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)<metadata>(.*?)</metadata>").unwrap());

/// Extract all metadata elements from an OAI-PMH response
///
/// Works for both GetRecord (single record) and ListRecords (multiple records).
/// Returns a vector of metadata XML strings in document order.
///
/// Uses regex pattern matching to extract content between `<metadata>` and `</metadata>` tags.
/// This is simpler than full XML parsing and works reliably for well-formed OAI-PMH responses.
///
/// # Example
/// ```
/// use oai_pmh::client::metadata::extract_metadata;
///
/// let xml = r#"
/// <OAI-PMH>
///   <ListRecords>
///     <record>
///       <header><identifier>id1</identifier></header>
///       <metadata><dc>content1</dc></metadata>
///     </record>
///     <record>
///       <header><identifier>id2</identifier></header>
///       <metadata><dc>content2</dc></metadata>
///     </record>
///   </ListRecords>
/// </OAI-PMH>"#;
///
/// let results = extract_metadata(xml);
/// assert_eq!(results.len(), 2);
/// assert!(results[0].contains("content1"));
/// assert!(results[1].contains("content2"));
/// ```
pub fn extract_metadata(xml: &str) -> Vec<String> {
    METADATA_RE
        .captures_iter(xml)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metadata_single_record() {
        let xml = std::fs::read_to_string("tests/fixtures/get_record.xml")
            .expect("Failed to load fixture");

        let results = extract_metadata(&xml);
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("<ead"));
        assert!(results[0].contains("xmlns=\"urn:isbn:1-931666-22-9\""));
    }

    #[test]
    fn test_extract_metadata_list_records() {
        let xml = std::fs::read_to_string("tests/fixtures/list_records.xml")
            .expect("Failed to load fixture");

        let results = extract_metadata(&xml);

        // Should have multiple records
        assert!(results.len() > 1);

        // First record
        assert!(results[0].contains("<oai_dc:dc"));
        assert!(results[0].contains("xmlns:dc=\"http://purl.org/dc/elements/1.1/\""));

        // All results should have non-empty metadata
        for metadata in &results {
            assert!(!metadata.is_empty());
        }
    }

    #[test]
    fn test_extract_metadata_with_newlines() {
        let xml = r#"<OAI-PMH><ListRecords><record><header></header><metadata>
<ead xmlns="test">
  <content>here</content>
</ead>
</metadata></record></ListRecords></OAI-PMH>"#;

        let results = extract_metadata(xml);
        assert_eq!(results.len(), 1);
        assert!(results[0].contains("<ead"));
        assert!(results[0].contains("</ead>"));
    }

    #[test]
    fn test_extract_metadata_multiple_records() {
        let xml = r#"
        <OAI-PMH>
          <ListRecords>
            <record>
              <header><identifier>id1</identifier></header>
              <metadata><dc xmlns="http://purl.org/dc/elements/1.1/"><title>First</title></dc></metadata>
            </record>
            <record>
              <header><identifier>id2</identifier></header>
              <metadata><dc xmlns="http://purl.org/dc/elements/1.1/"><title>Second</title></dc></metadata>
            </record>
          </ListRecords>
        </OAI-PMH>"#;

        let results = extract_metadata(xml);
        assert_eq!(results.len(), 2);

        assert!(results[0].contains("First"));
        assert!(results[1].contains("Second"));
    }
}
