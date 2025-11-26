pub mod client;

#[derive(Clone, Copy)]
pub enum Verb {
    GetRecord,
    Identify,
    ListIdentifiers,
    ListMetaDataFormats,
    ListRecords,
    ListSets,
}

impl Verb {
    pub fn as_param(&self) -> &'static str {
        match self {
            Verb::GetRecord => "GetRecord",
            Verb::Identify => "Identify",
            Verb::ListIdentifiers => "ListIdentifiers",
            Verb::ListMetaDataFormats => "ListMetaDataFormats",
            Verb::ListRecords => "ListRecords",
            Verb::ListSets => "ListSets",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Verb;

    #[test]
    fn converts_verb_to_param_string() {
        let verb = Verb::GetRecord;
        assert_eq!("GetRecord", verb.as_param());
    }
}
