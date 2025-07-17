

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::core::list::List;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

type CrateCString = crate::core::string::CString;
#[repr(C)]
pub struct CSvqlRuntimeConfig {

    pub pat_module_name: CrateCString,
    pub pat_filename: CrateCString,

    pub verbose: bool,
    pub const_ports: bool,
    pub nodefaultswaps: bool,
    pub compat_pairs: List<(CrateCString, CrateCString)>,
    pub swap_ports: List<CrateCString>,
    pub perm_ports: List<(CrateCString, List<CrateCString>, List<CrateCString>)>,
    pub cell_attr: List<CrateCString>,
    pub wire_attr: List<CrateCString>,
    pub ignore_parameters: bool,
    pub ignore_param: List<(CrateCString, CrateCString)>
}

impl CSvqlRuntimeConfig {
    pub fn new() -> *mut Self {
        let config = CSvqlRuntimeConfig {
            pat_module_name: CrateCString::new(),
            pat_filename: CrateCString::new(),
            verbose: false,
            const_ports: false,
            nodefaultswaps: false,
            compat_pairs: List::new(),
            swap_ports: List::new(),
            perm_ports: List::new(),
            cell_attr: List::new(),
            wire_attr: List::new(),
            ignore_parameters: false,
            ignore_param: List::new(),
        };
        Box::into_raw(Box::new(config))
    }
}

impl Default for SvqlRuntimeConfig {
    fn default() -> Self {
        SvqlRuntimeConfig {

            // new fields
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

impl From<SvqlRuntimeConfig> for CSvqlRuntimeConfig {
    fn from(svql_runtime_config: SvqlRuntimeConfig) -> Self {
        
        let pat_module_name: CrateCString = CrateCString::from(svql_runtime_config.pat_module_name);
        let pat_filename: CrateCString = CrateCString::from(svql_runtime_config.pat_filename);

        let verbose = svql_runtime_config.verbose;
        let const_ports = svql_runtime_config.const_ports;
        let nodefaultswaps = svql_runtime_config.nodefaultswaps;
        let compat_pairs: List<(CrateCString, CrateCString)> = svql_runtime_config.compat_pairs.iter().map(|(first, second)| {
            (CrateCString::from(first), CrateCString::from(second))
        }).collect();

        let swap_ports: List<CrateCString> = svql_runtime_config.swap_ports.iter().map(|(key, values)| {
            let values_list: List<CrateCString> = values.iter().map(|v| CrateCString::from(v)).collect();
            CrateCString::from(key)
        }).collect();

        let perm_ports: List<(CrateCString, List<CrateCString>, List<CrateCString>)> = svql_runtime_config.perm_ports.iter().map(|(key, first_values, second_values)| {
            let first_values_list: List<CrateCString> = first_values.iter().map(|v| CrateCString::from(v)).collect();
            let second_values_list: List<CrateCString> = second_values.iter().map(|v| CrateCString::from(v)).collect();
            (CrateCString::from(key), first_values_list, second_values_list)
        }).collect();

        let cell_attr: List<CrateCString> = svql_runtime_config.cell_attr.iter().map(|attr| CrateCString::from(attr)).collect();
        let wire_attr: List<CrateCString> = svql_runtime_config.wire_attr.iter().map(|attr| CrateCString::from(attr)).collect();
        let ignore_parameters = svql_runtime_config.ignore_parameters;
        let ignore_param: List<(CrateCString, CrateCString)> = svql_runtime_config.ignore_param.iter().map(|(first, second)| {
            (CrateCString::from(first), CrateCString::from(second))
        }).collect();

        CSvqlRuntimeConfig {
            pat_module_name,
            pat_filename,
            verbose,
            const_ports,
            nodefaultswaps,
            compat_pairs,
            swap_ports,
            perm_ports,
            cell_attr,
            wire_attr,
            ignore_parameters,
            ignore_param,
        }
    }
}

impl From<CSvqlRuntimeConfig> for SvqlRuntimeConfig {
    fn from(c_svql_runtime_config: CSvqlRuntimeConfig) -> Self {
        let pat_module_name = c_svql_runtime_config.pat_module_name.into();
        let pat_filename = c_svql_runtime_config.pat_filename.into();

        let verbose = c_svql_runtime_config.verbose;
        let const_ports = c_svql_runtime_config.const_ports;
        let nodefaultswaps = c_svql_runtime_config.nodefaultswaps;

        let compat_pairs: Vec<(String, String)> = c_svql_runtime_config.compat_pairs.iter().map(|(first, second)| {
            (first.into(), second.into())
        }).collect();

        let swap_ports: Vec<(String, Vec<String>)> = c_svql_runtime_config.swap_ports.iter().map(|key| {
            (key.into(), Vec::new()) // Assuming values are not needed here
        }).collect();

        let perm_ports: Vec<(String, Vec<String>, Vec<String>)> = c_svql_runtime_config.perm_ports.iter().map(|(key, first_values, second_values)| {
            (key.into(), first_values.into(), second_values.into())
        }).collect();

        let cell_attr: Vec<String> = c_svql_runtime_config.cell_attr.iter().map(|attr| attr.into()).collect();
        let wire_attr: Vec<String> = c_svql_runtime_config.wire_attr.iter().map(|attr| attr.into()).collect();
        
        let ignore_parameters = c_svql_runtime_config.ignore_parameters;
        
        let ignore_param: Vec<(String, String)> = c_svql_runtime_config.ignore_param.iter().map(|(first, second)| {
            (first.into(), second.into())
        }).collect();

        SvqlRuntimeConfig {
            pat_module_name,
            pat_filename,
            verbose,
            const_ports,
            nodefaultswaps,
            compat_pairs,
            swap_ports,
            perm_ports,
            cell_attr,
            wire_attr,
            ignore_parameters,
            ignore_param,
        }    
    }
}

// C FFI functions
#[unsafe(no_mangle)]
pub extern "C" fn c_svql_runtime_config_to_json(c_svql_runtime_config: *const CSvqlRuntimeConfig) -> *mut c_char {
    if c_svql_runtime_config.is_null() {
        return std::ptr::null_mut();
    }
    
    unsafe {
        let rust_svql_runtime_config = SvqlRuntimeConfig::from(std::ptr::read(c_svql_runtime_config));
        match serde_json::to_string(&rust_svql_runtime_config) {
            Ok(json_string) => {
                match CString::new(json_string) {
                    Ok(c_string) => c_string.into_raw(),
                    Err(_) => std::ptr::null_mut(),
                }
            }
            Err(_) => std::ptr::null_mut(),
        }
    }
}

// #[unsafe(no_mangle)]
// pub extern "C" fn c_svql_runtime_config_from_json(json_str: *const c_char) -> *mut CSvqlRuntimeConfig {
//     if json_str.is_null() {
//         return std::ptr::null_mut();
//     }
    
//     unsafe {
//         let c_str = CStr::from_ptr(json_str);
//         let json_string = match c_str.to_str() {
//             Ok(s) => s,
//             Err(_) => return std::ptr::null_mut(),
//         };
        
//         match serde_json::from_str::<SvqlRuntimeConfig>(json_string) {
//             Ok(svql_runtime_config) => {
//                 let c_svql_runtime_config = CSvqlRuntimeConfig::from(svql_runtime_config);
//                 Box::into_raw(Box::new(c_svql_runtime_config))
//             }
//             Err(_) => std::ptr::null_mut(),
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub extern "C" fn free_c_svql_runtime_config(c_svql_runtime_config: *mut CSvqlRuntimeConfig) {
    
// }