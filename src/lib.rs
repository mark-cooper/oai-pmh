use std::fmt;

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

impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verb::GetRecord => write!(f, "GetRecord"),
            Verb::Identify => write!(f, "Identify"),
            Verb::ListIdentifiers => write!(f, "ListIdentifiers"),
            Verb::ListMetaDataFormats => write!(f, "ListMetaDataFormats"),
            Verb::ListRecords => write!(f, "ListRecords"),
            Verb::ListSets => write!(f, "ListSets"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Verb;

    #[test]
    fn converts_verb_to_param() {
        let verb = Verb::GetRecord;
        assert_eq!("GetRecord", verb.to_string());
    }
}
