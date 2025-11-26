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
            verb: verb.as_param().to_string(),
            args,
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRecordArgs {
    pub identifier: String,
    pub metadata_prefix: String,
}

#[cfg(test)]
mod tests {
    use crate::{
        Verb,
        client::query::{GetRecordArgs, Query},
    };

    #[test]
    fn construct_get_record_query() {
        let q = "verb=GetRecord&identifier=oai:archivesspace:/repositories/2/resources/2&metadataPrefix=oai_ead";
        let args = GetRecordArgs {
            identifier: "oai:archivesspace:/repositories/2/resources/2".to_string(),
            metadata_prefix: "oai_ead".to_string(),
        };

        let expected: Query<GetRecordArgs> = Query::new(Verb::GetRecord, args);
        let actual: Query<GetRecordArgs> = serde_qs::from_str(q).unwrap();
        assert_eq!(expected, actual);
    }
}
