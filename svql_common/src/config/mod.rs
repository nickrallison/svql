pub mod args_parse;

use serde::{Deserialize, Serialize};
use crate::core::list::List;
use crate::core::string::CrateCString;
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CStringList {
    pub items: List<CrateCString>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompatPair {
    pub item1: CrateCString,
    pub item2: CrateCString,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompatPairs {
    pub items: List<CompatPair>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwapPort {
    pub name: CrateCString,
    pub ports: CStringList,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwapPorts {
    pub items: List<SwapPort>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PermPort {
    pub name: CrateCString,
    pub ports: CStringList,
    pub wires: CStringList,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PermPorts {
    pub items: List<PermPort>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IgnoreParam {
    pub name: CrateCString,
    pub value: CrateCString,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IgnoreParamList {
    pub items: List<IgnoreParam>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub ignore_param: IgnoreParamList,
}

// ==================== Conversion Implementations ====================

impl From<&SvqlRuntimeConfig> for CSvqlRuntimeConfig {
    fn from(cfg: &SvqlRuntimeConfig) -> Self {
        CSvqlRuntimeConfig {
            pat_module_name: CrateCString::from(cfg.pat_module_name.as_str()),
            pat_filename: CrateCString::from(cfg.pat_filename.as_str()),
            verbose: cfg.verbose,
            const_ports: cfg.const_ports,
            nodefaultswaps: cfg.nodefaultswaps,
            compat_pairs: CompatPairs {
                items: cfg.compat_pairs.iter().map(|(a, b)| CompatPair {
                    item1: CrateCString::from(a.as_str()),
                    item2: CrateCString::from(b.as_str()),
                }).collect(),
            },
            swap_ports: SwapPorts {
                items: cfg.swap_ports.iter().map(|(name, ports)| SwapPort {
                    name: CrateCString::from(name.as_str()),
                    ports: CStringList {
                        items: ports.iter().map(|s| CrateCString::from(s.as_str())).collect(),
                    },
                }).collect(),
            },
            perm_ports: PermPorts {
                items: cfg.perm_ports.iter().map(|(name, ports, wires)| PermPort {
                    name: CrateCString::from(name.as_str()),
                    ports: CStringList {
                        items: ports.iter().map(|s| CrateCString::from(s.as_str())).collect(),
                    },
                    wires: CStringList {
                        items: wires.iter().map(|s| CrateCString::from(s.as_str())).collect(),
                    },
                }).collect(),
            },
            cell_attr: CStringList {
                items: cfg.cell_attr.iter().map(|s| CrateCString::from(s.as_str())).collect(),
            },
            wire_attr: CStringList {
                items: cfg.wire_attr.iter().map(|s| CrateCString::from(s.as_str())).collect(),
            },
            ignore_parameters: cfg.ignore_parameters,
            ignore_param: IgnoreParamList {
                items: cfg.ignore_param.iter().map(|(name, value)| IgnoreParam {
                    name: CrateCString::from(name.as_str()),
                    value: CrateCString::from(value.as_str()),
                }).collect(),
            },
        }
    }
}

impl From<&CSvqlRuntimeConfig> for SvqlRuntimeConfig {
    fn from(c: &CSvqlRuntimeConfig) -> Self {
        SvqlRuntimeConfig {
            pat_module_name: c.pat_module_name.as_str().to_string(),
            pat_filename: c.pat_filename.as_str().to_string(),
            verbose: c.verbose,
            const_ports: c.const_ports,
            nodefaultswaps: c.nodefaultswaps,
            compat_pairs: c.compat_pairs.items.as_slice().iter().map(|cp| (
                cp.item1.as_str().to_string(),
                cp.item2.as_str().to_string(),
            )).collect(),
            swap_ports: c.swap_ports.items.as_slice().iter().map(|sp| (
                sp.name.as_str().to_string(),
                sp.ports.items.as_slice().iter().map(|s| s.as_str().to_string()).collect(),
            )).collect(),
            perm_ports: c.perm_ports.items.as_slice().iter().map(|pp| (
                pp.name.as_str().to_string(),
                pp.ports.items.as_slice().iter().map(|s| s.as_str().to_string()).collect(),
                pp.wires.items.as_slice().iter().map(|s| s.as_str().to_string()).collect(),
            )).collect(),
            cell_attr: c.cell_attr.items.as_slice().iter().map(|s| s.as_str().to_string()).collect(),
            wire_attr: c.wire_attr.items.as_slice().iter().map(|s| s.as_str().to_string()).collect(),
            ignore_parameters: c.ignore_parameters,
            ignore_param: c.ignore_param.items.as_slice().iter().map(|ip| (
                ip.name.as_str().to_string(),
                ip.value.as_str().to_string(),
            )).collect(),
        }
    }
}

// ==================== FFI Functions ====================

#[unsafe(no_mangle)]
pub extern "C" fn svql_runtime_config_new() -> *mut CSvqlRuntimeConfig {
    Box::into_raw(Box::new(CSvqlRuntimeConfig::from(&SvqlRuntimeConfig::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn svql_runtime_config_default() -> *mut CSvqlRuntimeConfig {
    svql_runtime_config_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn svql_runtime_config_clone(cfg: &CSvqlRuntimeConfig) -> CSvqlRuntimeConfig {
    cfg.clone()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_runtime_config_destroy(cfg: *mut CSvqlRuntimeConfig) {
    if !cfg.is_null() {
        unsafe { let _ = Box::from_raw(cfg); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn svql_runtime_config_eq(a: &CSvqlRuntimeConfig, b: &CSvqlRuntimeConfig) -> bool {
    a == b
}

#[unsafe(no_mangle)]
pub extern "C" fn svql_runtime_config_debug_string(cfg: &CSvqlRuntimeConfig) -> CrateCString {
    let rust_cfg = SvqlRuntimeConfig::from(cfg);
    let s = format!("{:?}", rust_cfg);
    CrateCString::from(s.as_str())
}

#[unsafe(no_mangle)]
pub extern "C" fn svql_runtime_config_to_json(cfg: &CSvqlRuntimeConfig) -> CrateCString {
    let rust_cfg = SvqlRuntimeConfig::from(cfg);
    match serde_json::to_string(&rust_cfg) {
        Ok(json) => CrateCString::from(json.as_str()),
        Err(e) => panic!("Failed to serialize to JSON: {}", e),
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn make_sample_config() -> SvqlRuntimeConfig {
        SvqlRuntimeConfig {
            pat_module_name: "mod".to_string(),
            pat_filename: "file.sv".to_string(),
            verbose: true,
            const_ports: true,
            nodefaultswaps: false,
            compat_pairs: vec![("a".to_string(), "b".to_string())],
            swap_ports: vec![("x".to_string(), vec!["y".to_string(), "z".to_string()])],
            perm_ports: vec![("p".to_string(), vec!["q".to_string()], vec!["r".to_string()])],
            cell_attr: vec!["foo".to_string()],
            wire_attr: vec!["bar".to_string()],
            ignore_parameters: true,
            ignore_param: vec![("baz".to_string(), "qux".to_string())],
        }
    }

    #[test]
    fn test_roundtrip_conversion() {
        let orig = make_sample_config();
        let c = CSvqlRuntimeConfig::from(&orig);
        let back = SvqlRuntimeConfig::from(&c);
        assert_eq!(orig, back);
    }

    #[test]
    fn test_clone_and_eq() {
        let c1 = CSvqlRuntimeConfig::from(&make_sample_config());
        let c2 = c1.clone();
        assert_eq!(c1, c2);
        assert_ne!(&c1 as *const _, &c2 as *const _);
    }

    #[test]
    fn test_hash() {
        let c1 = CSvqlRuntimeConfig::from(&make_sample_config());
        let c2 = c1.clone();
        let mut set = HashSet::new();
        set.insert(c1);
        assert!(set.contains(&c2));
    }

    #[test]
    fn test_debug_string() {
        let c = CSvqlRuntimeConfig::from(&make_sample_config());
        let dbg = svql_runtime_config_debug_string(&c);
        let s = dbg.as_str();
        assert!(s.contains("pat_module_name"));
        drop(dbg);
    }

    #[test]
    fn test_ffi_lifecycle() {
        let c = svql_runtime_config_new();
        let c2 = svql_runtime_config_clone(&c);
        assert!(svql_runtime_config_eq(&c, &c2));
        let _ = svql_runtime_config_debug_string(&c);
        unsafe {
            svql_runtime_config_destroy(Box::into_raw(Box::new(c2)));
            svql_runtime_config_destroy(Box::into_raw(Box::new(c)));
        }
    }

    #[test]
    fn test_default() {
        let c = svql_runtime_config_default();
        let rust = SvqlRuntimeConfig::default();
        let back = SvqlRuntimeConfig::from(&c);
        assert_eq!(rust, back);
    }
}