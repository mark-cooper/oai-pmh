use serde::{Deserialize, Serialize};

use crate::Verb;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Query<T> {
    verb: String,
    #[serde(flatten)]
    args: T,
}

impl<T> Query<T> {
    pub fn new(verb: Verb, args: T) -> Self {
        Self {
            verb: verb.to_string(),
            args,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRecordArgs {
    identifier: String,
    metadata_prefix: String,
}

impl GetRecordArgs {
    pub fn new(identifier: impl Into<String>, metadata_prefix: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            metadata_prefix: metadata_prefix.into(),
        }
    }
}

// TODO: ISO8601 (consider chrono in the future for stricter from/until handling)
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRecordsArgs {
    metadata_prefix: String,
    from: Option<String>,
    until: Option<String>,
    set: Option<String>,
}

impl ListRecordsArgs {
    pub fn new(metadata_prefix: impl Into<String>) -> Self {
        Self {
            metadata_prefix: metadata_prefix.into(),
            from: None,
            until: None,
            set: None,
        }
    }

    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn until(mut self, until: impl Into<String>) -> Self {
        self.until = Some(until.into());
        self
    }

    pub fn set(mut self, set: impl Into<String>) -> Self {
        self.set = Some(set.into());
        self
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumableArgs {
    resumption_token: String,
}

impl ResumableArgs {
    pub fn new(resumption_token: impl Into<String>) -> Self {
        Self {
            resumption_token: resumption_token.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Verb,
        client::query::{GetRecordArgs, ListRecordsArgs, Query},
    };

    #[test]
    fn construct_get_record_query() {
        let q = "verb=GetRecord&identifier=oai:archivesspace:/repositories/2/resources/2&metadataPrefix=oai_ead";
        let args = GetRecordArgs::new("oai:archivesspace:/repositories/2/resources/2", "oai_ead");

        let query = Query::new(Verb::GetRecord, args);
        let from_qs = serde_qs::from_str(q).unwrap();
        assert_eq!(query, from_qs);
    }

    #[test]
    fn construct_identify_query() {
        let q = "verb=Identify";

        let query = Query::new(Verb::Identify, ());
        let from_qs = serde_qs::from_str(q).unwrap();
        assert_eq!(query, from_qs);
    }

    #[test]
    fn construct_list_records_query() {
        let q = "verb=ListRecords&metadataPrefix=oai_ead&from=2000-01-01&until=2025-01-01";
        let args = ListRecordsArgs::new("oai_ead")
            .from("2000-01-01")
            .until("2025-01-01");

        let query = Query::new(Verb::ListRecords, args);
        let from_qs = serde_qs::from_str(q).unwrap();
        assert_eq!(query, from_qs);
    }
}
