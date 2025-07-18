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

#[repr(C)]
pub struct CStringList {
    pub items: List<CrateCString>,
}

// ##########
#[repr(C)]
pub struct CompatPair {
    pub item1: CrateCString,
    pub item2: CrateCString,
}

#[repr(C)]
pub struct CompatPairs {
    pub items: List<CompatPair>,
}

// ##########
#[repr(C)]
pub struct SwapPort {
    pub name: CrateCString,
    pub ports: CStringList,
}

#[repr(C)]
pub struct SwapPorts {
    pub items: List<SwapPort>,
}

// ##########

#[repr(C)]
pub struct PermPort {
    pub name: CrateCString,
    pub ports: CStringList,
    pub wires: CStringList,
}

#[repr(C)]
pub struct PermPorts {
    pub items: List<PermPort>,
}

// ##########

#[repr(C)]
pub struct IgnoreParam {
    pub name: CrateCString,
    pub value: CrateCString,
}

#[repr(C)]
pub struct IgnoreParamList {
    pub items: List<IgnoreParam>,
}


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
    pub ignore_param: IgnoreParamList
}
#[unsafe(no_mangle)]
pub extern "C" fn svql_runtime_config_new() -> CSvqlRuntimeConfig {
    CSvqlRuntimeConfig {
        pat_module_name: CrateCString::default(),
        pat_filename: CrateCString::default(),
        verbose: false,
        const_ports: false,
        nodefaultswaps: false,
        compat_pairs: CompatPairs { items: List::new() },
        swap_ports: SwapPorts { items: List::new() },
        perm_ports: PermPorts { items: List::new() },
        cell_attr: CStringList { items: List::new() },
        wire_attr: CStringList { items: List::new() },
        ignore_parameters: false,
        ignore_param: IgnoreParamList { items: List::new() },
    }
}