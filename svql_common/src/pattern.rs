// pattern.rs

use crate::pattern::ffi::Pattern;

#[cxx::bridge]
pub mod ffi {

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Pattern {
        pub file_loc: String,
        pub in_ports: Vec<String>,
        pub out_ports: Vec<String>,
        pub inout_ports: Vec<String>,
    }

    extern "Rust" {
        fn pattern_into_json_string(cfg: &Pattern) -> String;
        fn pattern_from_json_string(json: &str) -> Pattern;
    }
}

fn pattern_into_json_string(cfg: &Pattern) -> String {
    serde_json::to_string(cfg).expect("Failed to serialize Pattern to JSON")
}

fn pattern_from_json_string(json: &str) -> Pattern {
    serde_json::from_str(json).expect("Failed to deserialize JSON to Pattern")
}
