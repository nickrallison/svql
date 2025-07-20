use serde::{Deserialize, Serialize};
use crate::mat::ffi::{QueryMatchList};

#[cxx::bridge]
mod ffi {

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct QueryMatchList {
        pub matches: Vec<QueryMatch>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct QueryMatch {
        pub port_map: Vec<StringPair>,
        pub cell_map: Vec<CellPair>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
    pub struct CellData {
        pub cell_name: String,
        pub cell_index: usize,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
    pub struct StringPair {
        pub needle: String,
        pub haystack: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
    pub struct CellPair {
        pub needle: CellData,
        pub haystack: CellData,
    }

    extern "Rust" {
        fn matchlist_into_json_string(cfg: &QueryMatchList) -> String;
        fn matchlist_from_json_string(json: &str) -> QueryMatchList;
    }
}

fn matchlist_into_json_string(cfg: &QueryMatchList) -> String {
    serde_json::to_string(cfg).expect("Failed to serialize QueryMatchList to JSON")
}

fn matchlist_from_json_string(json: &str) -> QueryMatchList {
    serde_json::from_str(json).expect("Failed to deserialize JSON to QueryMatchList")
}