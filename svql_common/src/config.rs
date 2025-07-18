use serde::{Deserialize, Serialize};
use crate::core::list::List;
use crate::core::string::CrateCString;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SvqlRuntimeConfig {
    pub pat_module_name: String,
    pub pat_filename: String,

    pub verbose: bool,
    pub const_ports: bool,
    pub nodefaultswaps: bool,
    pub compat_pairs: Vec<(String, String)>,
    pub swap_ports: Vec<(String, Vec<String>)>,
    pub perm_ports: Vec<(String, Vec<String>, Vec<String>)>,
    pub cell_attr: Vec<String>,
    pub wire_attr: Vec<String>,
    pub ignore_parameters: bool,
    pub ignore_param: Vec<(String, String)>,
}

impl Default for SvqlRuntimeConfig {
    fn default() -> Self {
        Self {
            pat_module_name: String::new(),
            pat_filename: String::new(),
            verbose: false,
            const_ports: false,
            nodefaultswaps: false,
            compat_pairs: Vec::new(),
            swap_ports: Vec::new(),
            perm_ports: Vec::new(),
            cell_attr: Vec::new(),
            wire_attr: Vec::new(),
            ignore_parameters: false,
            ignore_param: Vec::new(),
        }
    }
}

pub type CStringList = List<CrateCString>;
pub type CompatPairs = List<(CrateCString, CrateCString)>;
pub type SwapPorts = List<(CrateCString, List<CrateCString>)>;
pub type PermPorts = List<(CrateCString, List<CrateCString>, List<CrateCString>)>;
pub type IgnoreParam = List<(CrateCString, CrateCString)>;

#[repr(C)]
pub struct CSvqlRuntimeConfig {
    pub pat_module_name: CrateCString,
    pub pat_filename: CrateCString,

    pub verbose: bool,
    pub const_ports: bool,
    pub nodefaultswaps: bool,
    pub compat_pairs: CompatPairs,
    pub swap_ports: SwapPorts,
    pub perm_ports: PermPorts,
    pub cell_attr: CStringList,
    pub wire_attr: CStringList,
    pub ignore_parameters: bool,
    pub ignore_param: IgnoreParam
}

impl From<SvqlRuntimeConfig> for CSvqlRuntimeConfig {
    fn from(config: SvqlRuntimeConfig) -> Self {
        Self {
            pat_module_name: config.pat_module_name.into(),
            pat_filename: config.pat_filename.into(),
            verbose: config.verbose,
            const_ports: config.const_ports,
            nodefaultswaps: config.nodefaultswaps,
            compat_pairs: config
                .compat_pairs
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect(),
            swap_ports: config
                .swap_ports
                .into_iter()
                .map(|(a, b)| (a.into(), b.into_iter().map(|s| s.into()).collect()))
                .collect(),
            perm_ports: config
                .perm_ports
                .into_iter()
                .map(|(a, b, c)| {
                    (
                        a.into(),
                        b.into_iter().map(|s| s.into()).collect(),
                        c.into_iter().map(|s| s.into()).collect(),
                    )
                })
                .collect(),
            cell_attr: config.cell_attr.into_iter().map(|s| s.into()).collect(),
            wire_attr: config.wire_attr.into_iter().map(|s| s.into()).collect(),
            ignore_parameters: config.ignore_parameters,
            ignore_param: config
                .ignore_param
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect(),
        }
    }
}

impl From<CSvqlRuntimeConfig> for SvqlRuntimeConfig {
    fn from(config: CSvqlRuntimeConfig) -> Self {
        Self {
            pat_module_name: config.pat_module_name.into(),
            pat_filename: config.pat_filename.into(),
            verbose: config.verbose,
            const_ports: config.const_ports,
            nodefaultswaps: config.nodefaultswaps,
            compat_pairs: config
                .compat_pairs
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect(),
            swap_ports: config
                .swap_ports
                .into_iter()
                .map(|(a, b)| (a.into(), b.into_iter().map(|s| s.into()).collect()))
                .collect(),
            perm_ports: config
                .perm_ports
                .into_iter()
                .map(|(a, b, c)| {
                    (
                        a.into(),
                        b.into_iter().map(|s| s.into()).collect(),
                        c.into_iter().map(|s| s.into()).collect(),
                    )
                })
                .collect(),
            cell_attr: config.cell_attr.into_iter().map(|s| s.into()).collect(),
            wire_attr: config.wire_attr.into_iter().map(|s| s.into()).collect(),
            ignore_parameters: config.ignore_parameters,
            ignore_param: config
                .ignore_param
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect(),
        }
    }
}

impl Default for CSvqlRuntimeConfig {
    fn default() -> Self {
        SvqlRuntimeConfig::default().into()
    }
}

// The Drop trait is implicitly handled by the `List` and `CrateCString` fields.
// A custom Drop implementation is only needed if `CSvqlRuntimeConfig` itself
// directly manages memory that needs to be freed. In this case, since all
// heap-allocated data is owned by `List` and `CrateCString`, their individual
// `Drop` implementations will be called automatically when a `CSvqlRuntimeConfig`
// instance goes out of scope. This prevents memory leaks.

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a non-default SvqlRuntimeConfig for testing
    fn create_test_config() -> SvqlRuntimeConfig {
        SvqlRuntimeConfig {
            pat_module_name: "test_module".to_string(),
            pat_filename: "test_file.pat".to_string(),
            verbose: true,
            const_ports: true,
            nodefaultswaps: true,
            compat_pairs: vec![("a".to_string(), "b".to_string()), ("c".to_string(), "d".to_string())],
            swap_ports: vec![(
                "swap1".to_string(),
                vec!["s1".to_string(), "s2".to_string()],
            )],
            perm_ports: vec![(
                "perm1".to_string(),
                vec!["p1".to_string(), "p2".to_string()],
                vec!["q1".to_string(), "q2".to_string()],
            )],
            cell_attr: vec!["attr1".to_string(), "attr2".to_string()],
            wire_attr: vec!["w_attr1".to_string()],
            ignore_parameters: true,
            ignore_param: vec![("param1".to_string(), "val1".to_string())],
        }
    }

    #[test]
    fn test_default_conversion() {
        // Create a default Rust config
        let rust_config = SvqlRuntimeConfig::default();
        // Convert it to the C-compatible version
        let c_config: CSvqlRuntimeConfig = rust_config.clone().into();

        // Check that the simple fields are correct
        assert_eq!(c_config.pat_module_name.as_str(), "");
        assert_eq!(c_config.pat_filename.as_str(), "");
        assert!(!c_config.verbose);
        assert!(!c_config.const_ports);
        assert!(!c_config.nodefaultswaps);
        assert!(!c_config.ignore_parameters);

        // Check that all the lists are empty
        assert!(c_config.compat_pairs.is_empty());
        assert!(c_config.swap_ports.is_empty());
        assert!(c_config.perm_ports.is_empty());
        assert!(c_config.cell_attr.is_empty());
        assert!(c_config.wire_attr.is_empty());
        assert!(c_config.ignore_param.is_empty());

        // Convert back and check for equality
        let rust_config_back: SvqlRuntimeConfig = c_config.into();
        assert_eq!(rust_config, rust_config_back);
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Create a populated Rust config
        let rust_config = create_test_config();
        // Convert to C-compatible config
        let c_config: CSvqlRuntimeConfig = rust_config.clone().into();

        // --- Assertions on the C-compatible struct ---
        assert_eq!(c_config.pat_module_name.as_str(), "test_module");
        assert_eq!(c_config.pat_filename.as_str(), "test_file.pat");
        assert!(c_config.verbose);
        assert!(c_config.const_ports);
        assert!(c_config.nodefaultswaps);
        assert!(c_config.ignore_parameters);

        // compat_pairs
        assert_eq!(c_config.compat_pairs.len(), 2);
        assert_eq!(c_config.compat_pairs[0].0.as_str(), "a");
        assert_eq!(c_config.compat_pairs[0].1.as_str(), "b");
        assert_eq!(c_config.compat_pairs[1].0.as_str(), "c");
        assert_eq!(c_config.compat_pairs[1].1.as_str(), "d");

        // swap_ports
        assert_eq!(c_config.swap_ports.len(), 1);
        assert_eq!(c_config.swap_ports[0].0.as_str(), "swap1");
        assert_eq!(c_config.swap_ports[0].1.len(), 2);
        assert_eq!(c_config.swap_ports[0].1[0].as_str(), "s1");
        assert_eq!(c_config.swap_ports[0].1[1].as_str(), "s2");

        // perm_ports
        assert_eq!(c_config.perm_ports.len(), 1);
        assert_eq!(c_config.perm_ports[0].0.as_str(), "perm1");
        assert_eq!(c_config.perm_ports[0].1.len(), 2);
        assert_eq!(c_config.perm_ports[0].1[0].as_str(), "p1");
        assert_eq!(c_config.perm_ports[0].1[1].as_str(), "p2");
        assert_eq!(c_config.perm_ports[0].2.len(), 2);
        assert_eq!(c_config.perm_ports[0].2[0].as_str(), "q1");
        assert_eq!(c_config.perm_ports[0].2[1].as_str(), "q2");

        // cell_attr
        assert_eq!(c_config.cell_attr.len(), 2);
        assert_eq!(c_config.cell_attr[0].as_str(), "attr1");
        assert_eq!(c_config.cell_attr[1].as_str(), "attr2");

        // wire_attr
        assert_eq!(c_config.wire_attr.len(), 1);
        assert_eq!(c_config.wire_attr[0].as_str(), "w_attr1");

        // ignore_param
        assert_eq!(c_config.ignore_param.len(), 1);
        assert_eq!(c_config.ignore_param[0].0.as_str(), "param1");
        assert_eq!(c_config.ignore_param[0].1.as_str(), "val1");

        // Convert back to the Rust struct
        let rust_config_back: SvqlRuntimeConfig = c_config.into();

        // Assert that the round-tripped config is identical to the original
        assert_eq!(rust_config, rust_config_back);
    }

    #[test]
    fn test_drop_populated() {
        // This test ensures that a fully populated C-compatible struct
        // can be created and then dropped without causing memory errors
        // like leaks or double-frees. Running this under `miri` will
        // validate the memory safety of the Drop implementations of
        // CrateCString and List.
        let rust_config = create_test_config();
        let c_config: CSvqlRuntimeConfig = rust_config.into();
        // The `c_config` goes out of scope here, and its Drop implementation is called.
        // If there's an issue, miri will report it.
        drop(c_config);
    }

    #[test]
    fn test_drop_default() {
        // This test ensures that a default-initialized (and thus empty)
        // C-compatible struct can be dropped without error.
        let c_config = CSvqlRuntimeConfig::default();
        drop(c_config);
    }
}