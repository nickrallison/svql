use serde::{Deserialize, Serialize};
use crate::config::ffi::{CompatPair, IgnoreParam, PermPort, SvqlRuntimeConfig, SwapPort};

#[cxx::bridge]
mod ffi {

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
    }
}

// Free functions for cxx bridge
fn new_svql_runtime_config() -> SvqlRuntimeConfig {
    SvqlRuntimeConfig::default()
}

fn svql_runtime_config_into_json_string(cfg: &SvqlRuntimeConfig) -> String {
    serde_json::to_string(cfg).expect("Failed to serialize SvqlRuntimeConfig to JSON")
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
        PermPort { type_name, left, right }
    }
}

impl IgnoreParam {
    pub fn new(param_name: String, param_value: String) -> Self {
        IgnoreParam { param_name, param_value }
    }
}