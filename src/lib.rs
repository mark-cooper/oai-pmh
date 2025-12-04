//! oai-pmh
//!
//! Rust library for the [Open Archives Initiative Protocol for Metadata Harvesting](https://www.openarchives.org/OAI/openarchivesprotocol.html).

use std::fmt;

pub mod client;
pub use client::Client;
pub use client::query::*;

#[derive(Clone, Copy)]
pub enum Verb {
    GetRecord,
    Identify,
    ListIdentifiers,
    ListMetadataFormats,
    ListRecords,
    ListSets,
}

impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verb::GetRecord => write!(f, "GetRecord"),
            Verb::Identify => write!(f, "Identify"),
            Verb::ListIdentifiers => write!(f, "ListIdentifiers"),
            Verb::ListMetadataFormats => write!(f, "ListMetadataFormats"),
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
