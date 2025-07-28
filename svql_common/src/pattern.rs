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

// Public functions for testing
pub fn pattern_into_json_string_public(cfg: &Pattern) -> String {
    pattern_into_json_string(cfg)
}

pub fn pattern_from_json_string_public(json: &str) -> Pattern {
    pattern_from_json_string(json)
}

#[cfg(test)]
mod tests {
    use crate::pattern::ffi::Pattern;
    use crate::pattern::{pattern_into_json_string_public, pattern_from_json_string_public};

    #[test]
    fn test_pattern_json_serialization() {
        let pattern = Pattern {
            file_loc: "/path/to/pattern.v".to_string(),
            in_ports: vec!["clk".to_string(), "reset".to_string(), "data_in".to_string()],
            out_ports: vec!["data_out".to_string(), "valid".to_string()],
            inout_ports: vec!["sda".to_string(), "scl".to_string()],
        };


        let serialized_expected = r#"{"file_loc":"/path/to/pattern.v","in_ports":["clk","reset","data_in"],"out_ports":["data_out","valid"],"inout_ports":["sda","scl"]}"#;
        let serialized_result = pattern_into_json_string_public(&pattern);
        assert_eq!(serialized_result, serialized_expected);

        let deserialized_result: Pattern = pattern_from_json_string_public(&serialized_result);
        assert_eq!(deserialized_result, pattern);
    }

    #[test]
    fn test_pattern_empty_ports() {
        let pattern = Pattern {
            file_loc: "empty.v".to_string(),
            in_ports: vec![],
            out_ports: vec![],
            inout_ports: vec![],
        };

        let serialized_expected = r#"{"file_loc":"empty.v","in_ports":[],"out_ports":[],"inout_ports":[]}"#;
        let serialized_result = pattern_into_json_string_public(&pattern);
        assert_eq!(serialized_result, serialized_expected);

        let deserialized_result: Pattern = pattern_from_json_string_public(&serialized_result);
        assert_eq!(deserialized_result, pattern);
    }

    #[test]
    #[should_panic(expected = "Failed to deserialize JSON to Pattern")]
    fn test_pattern_invalid_json() {
        pattern_from_json_string_public("invalid json string");
    }

    #[test]
    fn test_pattern_special_characters_in_file_path() {
        let pattern = Pattern {
            file_loc: "/home/user/projects/test-project_v2/pattern (copy).v".to_string(),
            in_ports: vec!["input".to_string()],
            out_ports: vec!["output".to_string()],
            inout_ports: vec![],
        };

        let serialzied_expected = r#"{"file_loc":"/home/user/projects/test-project_v2/pattern (copy).v","in_ports":["input"],"out_ports":["output"],"inout_ports":[]}"#;
        let serialized_result = pattern_into_json_string_public(&pattern);
        assert_eq!(serialized_result, serialzied_expected);

        let deserialized_result: Pattern = pattern_from_json_string_public(&serialized_result);
        assert_eq!(deserialized_result, pattern);
    }

    #[test]
    fn test_pattern_special_characters_in_port_names() {
        let pattern = Pattern {
            file_loc: "special.v".to_string(),
            in_ports: vec![
                "clk_100MHz".to_string(),
                "reset_n".to_string(),
                "data_bus[31:0]".to_string(),
                "enable$internal".to_string(),
            ],
            out_ports: vec![
                "status[3:0]".to_string(),
                "ready_flag".to_string(),
            ],
            inout_ports: vec![
                "i2c_sda".to_string(),
                "gpio[7:0]".to_string(),
            ],
        };

        let serialized_expected = r#"{"file_loc":"special.v","in_ports":["clk_100MHz","reset_n","data_bus[31:0]","enable$internal"],"out_ports":["status[3:0]","ready_flag"],"inout_ports":["i2c_sda","gpio[7:0]"]}"#;
        let serialized_result = pattern_into_json_string_public(&pattern);
        assert_eq!(serialized_result, serialized_expected);

        let deserialized_result: Pattern = pattern_from_json_string_public(&serialized_result);
        assert_eq!(deserialized_result, pattern);
    }
}
