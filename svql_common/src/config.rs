#![allow(unused_imports)]
use crate::config::ffi::{CompatPair, IgnoreParam, PermPort, SvqlRuntimeConfig, SwapPort};
use serde::{Deserialize, Serialize};

#[cxx::bridge]
pub mod ffi {

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct SvqlRuntimeConfig {
        pub pat_module_name: String,
        pub pat_filename: String,

        pub verbose: bool,
        pub const_ports: bool,
        pub nodefaultswaps: bool,
        pub compat_pairs: Vec<CompatPair>,
        pub swap_ports: Vec<SwapPort>,
        pub perm_ports: Vec<PermPort>,
        pub cell_attr: Vec<String>,
        pub wire_attr: Vec<String>,
        pub ignore_params: bool,
        pub ignored_parameters: Vec<IgnoreParam>,
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct CompatPair {
        pub needle: String,
        pub haystack: String,
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct SwapPort {
        pub type_name: String,
        pub ports: Vec<String>,
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct PermPort {
        pub type_name: String,
        pub left: Vec<String>,
        pub right: Vec<String>,
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct IgnoreParam {
        pub param_name: String,
        pub param_value: String,
    }

    extern "Rust" {
        fn svql_runtime_config_into_json_string(cfg: &SvqlRuntimeConfig) -> String;
        fn svql_runtime_config_from_json_string(json: &str) -> SvqlRuntimeConfig;
    }
}

// Free functions for cxx bridge
pub fn new_svql_runtime_config() -> SvqlRuntimeConfig {
    SvqlRuntimeConfig::default()
}

pub fn svql_runtime_config_into_json_string(cfg: &SvqlRuntimeConfig) -> String {
    serde_json::to_string(cfg).expect("Failed to serialize SvqlRuntimeConfig to JSON")
}

pub fn svql_runtime_config_from_json_string(json: &str) -> SvqlRuntimeConfig {
    serde_json::from_str(json).expect("Failed to deserialize JSON to SvqlRuntimeConfig")
}

impl CompatPair {
    pub fn new(needle: String, haystack: String) -> Self {
        CompatPair { needle, haystack }
    }
}

impl SwapPort {
    pub fn new(type_name: String, ports: Vec<String>) -> Self {
        SwapPort { type_name, ports }
    }
}

impl PermPort {
    pub fn new(type_name: String, left: Vec<String>, right: Vec<String>) -> Self {
        PermPort {
            type_name,
            left,
            right,
        }
    }
}

impl IgnoreParam {
    pub fn new(param_name: String, param_value: String) -> Self {
        IgnoreParam {
            param_name,
            param_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_svql_runtime_config() {
        let config = new_svql_runtime_config();
        assert_eq!(config.pat_module_name, "");
        assert_eq!(config.pat_filename, "");
        assert!(!config.verbose);
        assert!(!config.const_ports);
        assert!(!config.nodefaultswaps);
        assert!(config.compat_pairs.is_empty());
        assert!(config.swap_ports.is_empty());
        assert!(config.perm_ports.is_empty());
        assert!(config.cell_attr.is_empty());
        assert!(config.wire_attr.is_empty());
        assert!(!config.ignore_params);
        assert!(config.ignored_parameters.is_empty());
    }

    #[test]
    fn test_svql_runtime_config_json_serialization() {
        let mut config = new_svql_runtime_config();
        config.pat_module_name = "test_module".to_string();
        config.pat_filename = "test.v".to_string();
        config.verbose = true;
        config.const_ports = true;
        config.compat_pairs.push(CompatPair::new(
            "needle".to_string(),
            "haystack".to_string(),
        ));

        let json_string = svql_runtime_config_into_json_string(&config);
        assert!(json_string.contains("test_module"));
        assert!(json_string.contains("test.v"));
        assert!(json_string.contains("true"));

        let deserialized_config = svql_runtime_config_from_json_string(&json_string);
        assert_eq!(deserialized_config.pat_module_name, config.pat_module_name);
        assert_eq!(deserialized_config.pat_filename, config.pat_filename);
        assert_eq!(deserialized_config.verbose, config.verbose);
        assert_eq!(deserialized_config.const_ports, config.const_ports);
        assert_eq!(deserialized_config.compat_pairs.len(), 1);
        assert_eq!(deserialized_config.compat_pairs[0].needle, "needle");
        assert_eq!(deserialized_config.compat_pairs[0].haystack, "haystack");
    }

    #[test]
    fn test_svql_runtime_config_complex_serialization() {
        let mut config = new_svql_runtime_config();
        config.pat_module_name = "complex_module".to_string();
        config.swap_ports.push(SwapPort::new(
            "AND".to_string(),
            vec!["A".to_string(), "B".to_string()],
        ));
        config.perm_ports.push(PermPort::new(
            "OR".to_string(),
            vec!["X".to_string()],
            vec!["Y".to_string()],
        ));
        config
            .ignored_parameters
            .push(IgnoreParam::new("WIDTH".to_string(), "8".to_string()));
        config.cell_attr.push("keep".to_string());
        config.wire_attr.push("dont_touch".to_string());

        let json_string = svql_runtime_config_into_json_string(&config);
        let deserialized_config = svql_runtime_config_from_json_string(&json_string);

        assert_eq!(deserialized_config.pat_module_name, "complex_module");
        assert_eq!(deserialized_config.swap_ports.len(), 1);
        assert_eq!(deserialized_config.swap_ports[0].type_name, "AND");
        assert_eq!(deserialized_config.swap_ports[0].ports, vec!["A", "B"]);
        assert_eq!(deserialized_config.perm_ports.len(), 1);
        assert_eq!(deserialized_config.perm_ports[0].type_name, "OR");
        assert_eq!(deserialized_config.ignored_parameters.len(), 1);
        assert_eq!(
            deserialized_config.ignored_parameters[0].param_name,
            "WIDTH"
        );
        assert_eq!(deserialized_config.cell_attr, vec!["keep"]);
        assert_eq!(deserialized_config.wire_attr, vec!["dont_touch"]);
    }

    #[test]
    #[should_panic(expected = "Failed to deserialize JSON to SvqlRuntimeConfig")]
    fn test_svql_runtime_config_invalid_json() {
        svql_runtime_config_from_json_string("invalid json");
    }

    #[test]
    fn test_svql_runtime_config_roundtrip_serialization() {
        let original_config = new_svql_runtime_config();

        let json_string = svql_runtime_config_into_json_string(&original_config);
        let roundtrip_config = svql_runtime_config_from_json_string(&json_string);

        // Verify roundtrip preserves all fields
        assert_eq!(
            original_config.pat_module_name,
            roundtrip_config.pat_module_name
        );
        assert_eq!(original_config.pat_filename, roundtrip_config.pat_filename);
        assert_eq!(original_config.verbose, roundtrip_config.verbose);
        assert_eq!(original_config.const_ports, roundtrip_config.const_ports);
        assert_eq!(
            original_config.nodefaultswaps,
            roundtrip_config.nodefaultswaps
        );
        assert_eq!(
            original_config.ignore_params,
            roundtrip_config.ignore_params
        );
    }
}
