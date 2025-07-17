

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvqlRuntimeConfig {

    // new fields
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

// C-compatible representation
#[repr(C)]
pub struct CSvqlRuntimeConfig {

    pub pat_module_name: *mut c_char,
    pub pat_filename: *mut c_char,

    pub verbose: bool,
    pub const_ports: bool,
    pub nodefaultswaps: bool,
    pub compat_pairs_ptr: *mut CStringPair,
    pub compat_pairs_len: usize,
    pub swap_ports_ptr: *mut CStringVec,
    pub swap_ports_len: usize,
    pub perm_ports_ptr: *mut CStringVecPair,
    pub perm_ports_len: usize,
    pub cell_attr_ptr: *mut *mut c_char,
    pub cell_attr_len: usize,
    pub wire_attr_ptr: *mut *mut c_char,
    pub wire_attr_len: usize,
    pub ignore_parameters: bool,
    pub ignore_param_ptr: *mut CStringPair,
    pub ignore_param_len: usize,
}

#[repr(C)]
pub struct CStringPair {
    pub first: *mut c_char,
    pub second: *mut c_char,
}

#[repr(C)]
pub struct CStringVec {
    pub key: *mut c_char,
    pub values_ptr: *mut *mut c_char,
    pub values_len: usize,
}

#[repr(C)]
pub struct CStringVecPair {
    pub key: *mut c_char,
    pub first_values_ptr: *mut *mut c_char,
    pub first_values_len: usize,
    pub second_values_ptr: *mut *mut c_char,
    pub second_values_len: usize,
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
        // Capture lengths before moving
        let compat_pairs_len = svql_runtime_config.compat_pairs.len();
        let swap_ports_len = svql_runtime_config.swap_ports.len();
        let perm_ports_len = svql_runtime_config.perm_ports.len();
        let cell_attr_len = svql_runtime_config.cell_attr.len();
        let wire_attr_len = svql_runtime_config.wire_attr.len();
        let ignore_param_len = svql_runtime_config.ignore_param.len();

        // Convert compat_pairs
        let compat_pairs: Vec<CStringPair> = svql_runtime_config
            .compat_pairs
            .into_iter()
            .map(|(first, second)| CStringPair {
                first: CString::new(first).unwrap().into_raw(),
                second: CString::new(second).unwrap().into_raw(),
            })
            .collect();
        let compat_pairs_ptr = if compat_pairs.is_empty() {
            std::ptr::null_mut()
        } else {
            Box::into_raw(compat_pairs.into_boxed_slice()) as *mut CStringPair
        };

        // Convert swap_ports
        let swap_ports: Vec<CStringVec> = svql_runtime_config
            .swap_ports
            .into_iter()
            .map(|(key, values)| {
                let values_len = values.len();
                let c_values: Vec<*mut c_char> = values
                    .into_iter()
                    .map(|s| CString::new(s).unwrap().into_raw())
                    .collect();
                let values_ptr = if c_values.is_empty() {
                    std::ptr::null_mut()
                } else {
                    Box::into_raw(c_values.into_boxed_slice()) as *mut *mut c_char
                };
                CStringVec {
                    key: CString::new(key).unwrap().into_raw(),
                    values_ptr,
                    values_len,
                }
            })
            .collect();
        let swap_ports_ptr = if swap_ports.is_empty() {
            std::ptr::null_mut()
        } else {
            Box::into_raw(swap_ports.into_boxed_slice()) as *mut CStringVec
        };

        // Convert perm_ports
        let perm_ports: Vec<CStringVecPair> = svql_runtime_config
            .perm_ports
            .into_iter()
            .map(|(key, first_values, second_values)| {
                let first_values_len = first_values.len();
                let second_values_len = second_values.len();
                
                let c_first_values: Vec<*mut c_char> = first_values
                    .into_iter()
                    .map(|s| CString::new(s).unwrap().into_raw())
                    .collect();
                let first_values_ptr = if c_first_values.is_empty() {
                    std::ptr::null_mut()
                } else {
                    Box::into_raw(c_first_values.into_boxed_slice()) as *mut *mut c_char
                };

                let c_second_values: Vec<*mut c_char> = second_values
                    .into_iter()
                    .map(|s| CString::new(s).unwrap().into_raw())
                    .collect();
                let second_values_ptr = if c_second_values.is_empty() {
                    std::ptr::null_mut()
                } else {
                    Box::into_raw(c_second_values.into_boxed_slice()) as *mut *mut c_char
                };

                CStringVecPair {
                    key: CString::new(key).unwrap().into_raw(),
                    first_values_ptr,
                    first_values_len,
                    second_values_ptr,
                    second_values_len,
                }
            })
            .collect();
        let perm_ports_ptr = if perm_ports.is_empty() {
            std::ptr::null_mut()
        } else {
            Box::into_raw(perm_ports.into_boxed_slice()) as *mut CStringVecPair
        };

        // Convert cell_attr
        let cell_attr: Vec<*mut c_char> = svql_runtime_config
            .cell_attr
            .into_iter()
            .map(|s| CString::new(s).unwrap().into_raw())
            .collect();
        let cell_attr_ptr = if cell_attr.is_empty() {
            std::ptr::null_mut()
        } else {
            Box::into_raw(cell_attr.into_boxed_slice()) as *mut *mut c_char
        };

        // Convert wire_attr
        let wire_attr: Vec<*mut c_char> = svql_runtime_config
            .wire_attr
            .into_iter()
            .map(|s| CString::new(s).unwrap().into_raw())
            .collect();
        let wire_attr_ptr = if wire_attr.is_empty() {
            std::ptr::null_mut()
        } else {
            Box::into_raw(wire_attr.into_boxed_slice()) as *mut *mut c_char
        };

        // Convert ignore_param
        let ignore_param: Vec<CStringPair> = svql_runtime_config
            .ignore_param
            .into_iter()
            .map(|(first, second)| CStringPair {
                first: CString::new(first).unwrap().into_raw(),
                second: CString::new(second).unwrap().into_raw(),
            })
            .collect();
        let ignore_param_ptr = if ignore_param.is_empty() {
            std::ptr::null_mut()
        } else {
            Box::into_raw(ignore_param.into_boxed_slice()) as *mut CStringPair
        };

        CSvqlRuntimeConfig {
            verbose: svql_runtime_config.verbose,
            const_ports: svql_runtime_config.const_ports,
            nodefaultswaps: svql_runtime_config.nodefaultswaps,
            compat_pairs_ptr,
            compat_pairs_len,
            swap_ports_ptr,
            swap_ports_len,
            perm_ports_ptr,
            perm_ports_len,
            cell_attr_ptr,
            cell_attr_len,
            wire_attr_ptr,
            wire_attr_len,
            ignore_parameters: svql_runtime_config.ignore_parameters,
            ignore_param_ptr,
            ignore_param_len,
        }
    }
}

impl From<CSvqlRuntimeConfig> for SvqlRuntimeConfig {
    fn from(c_svql_runtime_config: CSvqlRuntimeConfig) -> Self {
        unsafe {
            // Convert compat_pairs
            let compat_pairs = if c_svql_runtime_config.compat_pairs_ptr.is_null() {
                Vec::new()
            } else {
                let slice = std::slice::from_raw_parts(c_svql_runtime_config.compat_pairs_ptr, c_svql_runtime_config.compat_pairs_len);
                slice
                    .iter()
                    .map(|pair| {
                        let first = CStr::from_ptr(pair.first).to_string_lossy().into_owned();
                        let second = CStr::from_ptr(pair.second).to_string_lossy().into_owned();
                        (first, second)
                    })
                    .collect()
            };

            // Convert swap_ports
            let swap_ports = if c_svql_runtime_config.swap_ports_ptr.is_null() {
                Vec::new()
            } else {
                let slice = std::slice::from_raw_parts(c_svql_runtime_config.swap_ports_ptr, c_svql_runtime_config.swap_ports_len);
                slice
                    .iter()
                    .map(|item| {
                        let key = CStr::from_ptr(item.key).to_string_lossy().into_owned();
                        let values = if item.values_ptr.is_null() {
                            Vec::new()
                        } else {
                            let values_slice = std::slice::from_raw_parts(item.values_ptr, item.values_len);
                            values_slice
                                .iter()
                                .map(|ptr| CStr::from_ptr(*ptr).to_string_lossy().into_owned())
                                .collect()
                        };
                        (key, values)
                    })
                    .collect()
            };

            // Convert perm_ports
            let perm_ports = if c_svql_runtime_config.perm_ports_ptr.is_null() {
                Vec::new()
            } else {
                let slice = std::slice::from_raw_parts(c_svql_runtime_config.perm_ports_ptr, c_svql_runtime_config.perm_ports_len);
                slice
                    .iter()
                    .map(|item| {
                        let key = CStr::from_ptr(item.key).to_string_lossy().into_owned();
                        let first_values = if item.first_values_ptr.is_null() {
                            Vec::new()
                        } else {
                            let first_slice = std::slice::from_raw_parts(item.first_values_ptr, item.first_values_len);
                            first_slice
                                .iter()
                                .map(|ptr| CStr::from_ptr(*ptr).to_string_lossy().into_owned())
                                .collect()
                        };
                        let second_values = if item.second_values_ptr.is_null() {
                            Vec::new()
                        } else {
                            let second_slice = std::slice::from_raw_parts(item.second_values_ptr, item.second_values_len);
                            second_slice
                                .iter()
                                .map(|ptr| CStr::from_ptr(*ptr).to_string_lossy().into_owned())
                                .collect()
                        };
                        (key, first_values, second_values)
                    })
                    .collect()
            };

            // Convert cell_attr
            let cell_attr = if c_svql_runtime_config.cell_attr_ptr.is_null() {
                Vec::new()
            } else {
                let slice = std::slice::from_raw_parts(c_svql_runtime_config.cell_attr_ptr, c_svql_runtime_config.cell_attr_len);
                slice
                    .iter()
                    .map(|ptr| CStr::from_ptr(*ptr).to_string_lossy().into_owned())
                    .collect()
            };

            // Convert wire_attr
            let wire_attr = if c_svql_runtime_config.wire_attr_ptr.is_null() {
                Vec::new()
            } else {
                let slice = std::slice::from_raw_parts(c_svql_runtime_config.wire_attr_ptr, c_svql_runtime_config.wire_attr_len);
                slice
                    .iter()
                    .map(|ptr| CStr::from_ptr(*ptr).to_string_lossy().into_owned())
                    .collect()
            };

            // Convert ignore_param
            let ignore_param = if c_svql_runtime_config.ignore_param_ptr.is_null() {
                Vec::new()
            } else {
                let slice = std::slice::from_raw_parts(c_svql_runtime_config.ignore_param_ptr, c_svql_runtime_config.ignore_param_len);
                slice
                    .iter()
                    .map(|pair| {
                        let first = CStr::from_ptr(pair.first).to_string_lossy().into_owned();
                        let second = CStr::from_ptr(pair.second).to_string_lossy().into_owned();
                        (first, second)
                    })
                    .collect()
            };

            SvqlRuntimeConfig {
                verbose: c_svql_runtime_config.verbose,
                const_ports: c_svql_runtime_config.const_ports,
                nodefaultswaps: c_svql_runtime_config.nodefaultswaps,
                compat_pairs,
                swap_ports,
                perm_ports,
                cell_attr,
                wire_attr,
                ignore_parameters: c_svql_runtime_config.ignore_parameters,
                ignore_param,
            }
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

#[unsafe(no_mangle)]
pub extern "C" fn c_svql_runtime_config_from_json(json_str: *const c_char) -> *mut CSvqlRuntimeConfig {
    if json_str.is_null() {
        return std::ptr::null_mut();
    }
    
    unsafe {
        let c_str = CStr::from_ptr(json_str);
        let json_string = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        
        match serde_json::from_str::<SvqlRuntimeConfig>(json_string) {
            Ok(svql_runtime_config) => {
                let c_svql_runtime_config = CSvqlRuntimeConfig::from(svql_runtime_config);
                Box::into_raw(Box::new(c_svql_runtime_config))
            }
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_c_svql_runtime_config(c_svql_runtime_config: *mut CSvqlRuntimeConfig) {
    if c_svql_runtime_config.is_null() {
        return;
    }

    if !config.compat_pairs_ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(config.compat_pairs_ptr, config.compat_pairs_len) };
        for pair in slice {
            if !pair.first.is_null() {
                let _ = unsafe { CString::from_raw(pair.first) };
            }
            if !pair.second.is_null() {
                let _ = unsafe { CString::from_raw(pair.second) };
            }
        }
        let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(config.compat_pairs_ptr, config.compat_pairs_len)) };
    }

    // Free swap_ports
    if !config.swap_ports_ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(config.swap_ports_ptr, config.swap_ports_len) };
        for item in slice {
            if !item.key.is_null() {
                let _ = unsafe { CString::from_raw(item.key) };
            }
            if !item.values_ptr.is_null() {
                let values_slice = unsafe { std::slice::from_raw_parts(item.values_ptr, item.values_len) };
                for ptr in values_slice {
                    if !ptr.is_null() {
                        let _ = unsafe { CString::from_raw(*ptr) };
                    }
                }
                let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(item.values_ptr, item.values_len)) };
            }
        }
        let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(config.swap_ports_ptr, config.swap_ports_len)) };
    }

    // Free perm_ports
    if !config.perm_ports_ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(config.perm_ports_ptr, config.perm_ports_len) };
        for item in slice {
            if !item.key.is_null() {
                let _ = unsafe { CString::from_raw(item.key) };
            }
            if !item.first_values_ptr.is_null() {
                let first_slice = unsafe { std::slice::from_raw_parts(item.first_values_ptr, item.first_values_len) };
                for ptr in first_slice {
                    if !ptr.is_null() {
                        let _ = unsafe { CString::from_raw(*ptr) };
                    }
                }
                let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(item.first_values_ptr, item.first_values_len)) };
            }
            if !item.second_values_ptr.is_null() {
                let second_slice = unsafe { std::slice::from_raw_parts(item.second_values_ptr, item.second_values_len) };
                for ptr in second_slice {
                    if !ptr.is_null() {
                        let _ = unsafe { CString::from_raw(*ptr) };
                    }
                }
                let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(item.second_values_ptr, item.second_values_len)) };
            }
        }
        let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(config.perm_ports_ptr, config.perm_ports_len)) };
    }

    // Free cell_attr
    if !config.cell_attr_ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(config.cell_attr_ptr, config.cell_attr_len) };
        for ptr in slice {
            if !ptr.is_null() {
                let _ = unsafe { CString::from_raw(*ptr) };
            }
        }
        let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(config.cell_attr_ptr, config.cell_attr_len)) };
    }

    // Free wire_attr
    if !config.wire_attr_ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(config.wire_attr_ptr, config.wire_attr_len) };
        for ptr in slice {
            if !ptr.is_null() {
                let _ = unsafe { CString::from_raw(*ptr) };
            }
        }
        let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(config.wire_attr_ptr, config.wire_attr_len)) } ;
    }
    
    // Free ignore_param
    if !config.ignore_param_ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(config.ignore_param_ptr, config.ignore_param_len) };
        for pair in slice {
            if !pair.first.is_null() {
                let _ = unsafe { CString::from_raw(pair.first) };
            }
            if !pair.second.is_null() {
                let _ = unsafe { CString::from_raw(pair.second) };
            }
        }
        let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(config.ignore_param_ptr, config.ignore_param_len)) };
    }
}