use crate::error::{Error, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

fn local_name(name: &[u8]) -> &[u8] {
    name.split(|b| *b == b':').next_back().unwrap_or(name)
}

pub(crate) fn extract_record_metadata(xml: &str) -> Result<Vec<Option<String>>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut skip_buf = Vec::new();
    let mut stack: Vec<Vec<u8>> = Vec::new();

    let mut payload_depth: Option<usize> = None;
    let mut record_depth: Option<usize> = None;
    let mut current_record: Option<usize> = None;

    let mut results: Vec<Option<String>> = Vec::new();

    loop {
        let event = match reader.read_event_into(&mut buf) {
            Ok(event) => event,
            Err(err) => return Err(Error::XmlParse(err.into())),
        };

        match event {
            Event::Start(e) => {
                let name = local_name(e.name().into_inner()).to_vec();
                stack.push(name.clone());

                match name.as_slice() {
                    b"GetRecord" | b"ListRecords" if payload_depth.is_none() => {
                        payload_depth = Some(stack.len());
                    }
                    b"record"
                        if payload_depth.is_some()
                            && record_depth.is_none()
                            && stack.len() == payload_depth.unwrap() + 1 =>
                    {
                        record_depth = Some(stack.len());
                        let idx = results.len();
                        results.push(None);
                        current_record = Some(idx);
                    }
                    b"metadata"
                        if current_record.is_some()
                            && record_depth.is_some()
                            && stack.len() == record_depth.unwrap() + 1 =>
                    {
                        let span = reader
                            .read_to_end_into(e.name(), &mut skip_buf)
                            .map_err(|err| Error::XmlParse(err.into()))?;
                        let start = span.start as usize;
                        let end = span.end as usize;
                        let inner_xml = xml.get(start..end).unwrap_or_default().to_string();

                        if let Some(idx) = current_record {
                            results[idx] = Some(inner_xml);
                        }

                        // `read_to_end_into` consumes the corresponding `End` event, so we need to
                        // close this element in our own stack tracking.
                        stack.pop();
                    }
                    _ => (),
                }
            }
            Event::Empty(e) => {
                let name = local_name(e.name().into_inner());

                if name == b"metadata"
                    && record_depth.is_some()
                    && stack.len() == record_depth.unwrap()
                    && let Some(idx) = current_record
                {
                    results[idx] = Some(String::new());
                }
            }
            Event::End(e) => {
                let name = local_name(e.name().into_inner());

                if matches!(name, b"GetRecord" | b"ListRecords")
                    && payload_depth.is_some()
                    && stack.len() == payload_depth.unwrap()
                {
                    payload_depth = None;
                }

                if name == b"record"
                    && record_depth.is_some()
                    && stack.len() == record_depth.unwrap()
                {
                    record_depth = None;
                    current_record = None;
                }

                stack.pop();
            }
            Event::Eof => break,
            _ => (),
        }

        buf.clear();
    }

    Ok(results)
}

/// Extract all metadata elements from an OAI-PMH response
///
/// Works for both GetRecord (single record) and ListRecords (multiple records).
/// Returns a vector of metadata XML strings in document order.
///
/// Uses a streaming XML reader to extract the inner XML of each `<metadata>` element (without
/// attempting to parse the metadata itself).
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
    extract_record_metadata(xml)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
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

    #[test]
    fn test_extract_record_metadata_alignment_with_deleted() {
        let xml = std::fs::read_to_string("tests/fixtures/list_records_with_deleted.xml")
            .expect("Failed to load fixture");

        let results = extract_record_metadata(&xml).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], None);

        let second = results[1].as_ref().unwrap();
        assert!(second.contains("<oai_dc:dc"));
        assert!(second.contains("<dc:title>Alive</dc:title>"));
    }
}
