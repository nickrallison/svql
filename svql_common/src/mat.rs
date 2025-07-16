use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

/// Represents a match in the source code.
/// Includes the source location and the ports involved in the match.
/// Where the keys of each port are the name of the signals specified in the pattern,
/// and the values are the names of the signals in the source code that match the pattern.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Match {
    pub port_map: HashMap<String, String>,
    pub cell_map: HashMap<CellData, CellData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct CellData {
    pub cell_name: String,
    pub cell_index: usize,
}

/// C FFI equivalent of CellData
#[repr(C)]
pub struct CCellData {
    pub cell_name: *mut c_char,
    pub cell_index: usize,
}

impl CCellData {
    /// Convert from Rust CellData to C FFI CellData
    pub fn from_rust(cell_data: &CellData) -> Self {
        let cell_name = CString::new(cell_data.cell_name.clone())
            .unwrap_or_else(|_| CString::new("").unwrap())
            .into_raw();

        CCellData {
            cell_name,
            cell_index: cell_data.cell_index,
        }
    }

    /// Convert from C FFI CellData to Rust CellData
    pub fn to_rust(&self) -> CellData {
        let cell_name = if self.cell_name.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(self.cell_name).to_string_lossy().to_string() }
        };

        CellData {
            cell_name,
            cell_index: self.cell_index,
        }
    }
}

/// C FFI equivalent of Match
#[repr(C)]
pub struct CMatch {
    pub port_map: *mut CStringMap,
    pub cell_map: *mut CCellDataMap,
}

/// C FFI representation of a string-to-string map entry
#[repr(C)]
pub struct CStringMapEntry {
    pub key: *mut c_char,
    pub value: *mut c_char,
}

/// C FFI representation of a string-to-string map
#[repr(C)]
pub struct CStringMap {
    pub entries: *mut CStringMapEntry,
    pub len: usize,
}

/// C FFI representation of a CellData-to-CellData map entry
#[repr(C)]
pub struct CCellDataMapEntry {
    pub key: CCellData,
    pub value: CCellData,
}

/// C FFI representation of a CellData-to-CellData map
#[repr(C)]
pub struct CCellDataMap {
    pub entries: *mut CCellDataMapEntry,
    pub len: usize,
}

impl CMatch {
    /// Convert from Rust Match to C FFI Match
    pub fn from_rust(match_data: &Match) -> Self {
        // Convert port_map
        let port_entries: Vec<CStringMapEntry> = match_data
            .port_map
            .iter()
            .map(|(k, v)| {
                let key = CString::new(k.clone())
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw();
                let value = CString::new(v.clone())
                    .unwrap_or_else(|_| CString::new("").unwrap())
                    .into_raw();
                CStringMapEntry { key, value }
            })
            .collect();

        let port_map = Box::into_raw(Box::new(CStringMap {
            entries: if port_entries.is_empty() {
                ptr::null_mut()
            } else {
                port_entries.as_ptr() as *mut CStringMapEntry
            },
            len: port_entries.len(),
        }));
        std::mem::forget(port_entries);

        // Convert cell_map
        let cell_entries: Vec<CCellDataMapEntry> = match_data
            .cell_map
            .iter()
            .map(|(k, v)| CCellDataMapEntry {
                key: CCellData::from_rust(k),
                value: CCellData::from_rust(v),
            })
            .collect();

        let cell_map = Box::into_raw(Box::new(CCellDataMap {
            entries: if cell_entries.is_empty() {
                ptr::null_mut()
            } else {
                cell_entries.as_ptr() as *mut CCellDataMapEntry
            },
            len: cell_entries.len(),
        }));
        std::mem::forget(cell_entries);

        CMatch { port_map, cell_map }
    }

    /// Convert from C FFI Match to Rust Match
    pub fn to_rust(&self) -> Match {
        let mut port_map = HashMap::new();
        let mut cell_map = HashMap::new();

        // Convert port_map
        if !self.port_map.is_null() {
            unsafe {
                let port_map_ref = &*self.port_map;
                if !port_map_ref.entries.is_null() {
                    let entries =
                        std::slice::from_raw_parts(port_map_ref.entries, port_map_ref.len);
                    for entry in entries {
                        let key = if entry.key.is_null() {
                            String::new()
                        } else {
                            CStr::from_ptr(entry.key).to_string_lossy().to_string()
                        };
                        let value = if entry.value.is_null() {
                            String::new()
                        } else {
                            CStr::from_ptr(entry.value).to_string_lossy().to_string()
                        };
                        port_map.insert(key, value);
                    }
                }
            }
        }

        // Convert cell_map
        if !self.cell_map.is_null() {
            unsafe {
                let cell_map_ref = &*self.cell_map;
                if !cell_map_ref.entries.is_null() {
                    let entries =
                        std::slice::from_raw_parts(cell_map_ref.entries, cell_map_ref.len);
                    for entry in entries {
                        let key = entry.key.to_rust();
                        let value = entry.value.to_rust();
                        cell_map.insert(key, value);
                    }
                }
            }
        }

        Match { port_map, cell_map }
    }
}

// C FFI functions for CCellData

/// Create a new CCellData
#[unsafe(no_mangle)]
pub extern "C" fn ccelldata_new(cell_name: *const c_char, cell_index: usize) -> *mut CCellData {
    let cell_name_str = if cell_name.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(cell_name).to_string_lossy().to_string() }
    };

    let cell_name_cstring = CString::new(cell_name_str)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw();

    let ccell_data = CCellData {
        cell_name: cell_name_cstring,
        cell_index,
    };

    Box::into_raw(Box::new(ccell_data))
}

/// Serialize CCellData to JSON C string
#[unsafe(no_mangle)]
pub extern "C" fn ccelldata_serialize(ccell_data: *const CCellData) -> *mut c_char {
    if ccell_data.is_null() {
        return ptr::null_mut();
    }

    let cell_data = unsafe { (*ccell_data).to_rust() };

    match serde_json::to_string(&cell_data) {
        Ok(json) => CString::new(json)
            .unwrap_or_else(|_| CString::new("{}").unwrap())
            .into_raw(),
        Err(_) => CString::new("{}").unwrap().into_raw(),
    }
}

/// Free CCellData memory
#[unsafe(no_mangle)]
pub extern "C" fn ccelldata_free(ccell_data: *mut CCellData) {
    if ccell_data.is_null() {
        return;
    }

    unsafe {
        // Free the cell_name string
        if !(*ccell_data).cell_name.is_null() {
            let _ = CString::from_raw((*ccell_data).cell_name);
        }
        // Free the CCellData struct
        let _ = Box::from_raw(ccell_data);
    }
}

// C FFI functions for CMatch

/// Create a new CMatch
#[unsafe(no_mangle)]
pub extern "C" fn cmatch_new() -> *mut CMatch {
    let cmatch = CMatch {
        port_map: Box::into_raw(Box::new(CStringMap {
            entries: ptr::null_mut(),
            len: 0,
        })),
        cell_map: Box::into_raw(Box::new(CCellDataMap {
            entries: ptr::null_mut(),
            len: 0,
        })),
    };

    Box::into_raw(Box::new(cmatch))
}

/// Serialize CMatch to JSON C string
#[unsafe(no_mangle)]
pub extern "C" fn cmatch_serialize(cmatch: *const CMatch) -> *mut c_char {
    if cmatch.is_null() {
        return ptr::null_mut();
    }

    let match_data = unsafe { (*cmatch).to_rust() };

    match serde_json::to_string(&match_data) {
        Ok(json) => CString::new(json)
            .unwrap_or_else(|_| CString::new("{}").unwrap())
            .into_raw(),
        Err(_) => CString::new("{}").unwrap().into_raw(),
    }
}

/// Free CMatch memory
#[unsafe(no_mangle)]
pub extern "C" fn cmatch_free(cmatch: *mut CMatch) {
    if cmatch.is_null() {
        return;
    }

    unsafe {
        let cmatch_ref = &*cmatch;

        // Free port_map
        if !cmatch_ref.port_map.is_null() {
            let port_map_ref = &*cmatch_ref.port_map;
            if !port_map_ref.entries.is_null() {
                let entries =
                    std::slice::from_raw_parts_mut(port_map_ref.entries, port_map_ref.len);
                for entry in entries {
                    if !entry.key.is_null() {
                        let _ = CString::from_raw(entry.key);
                    }
                    if !entry.value.is_null() {
                        let _ = CString::from_raw(entry.value);
                    }
                }
                let _ =
                    Vec::from_raw_parts(port_map_ref.entries, port_map_ref.len, port_map_ref.len);
            }
            let _ = Box::from_raw(cmatch_ref.port_map);
        }

        // Free cell_map
        if !cmatch_ref.cell_map.is_null() {
            let cell_map_ref = &*cmatch_ref.cell_map;
            if !cell_map_ref.entries.is_null() {
                let entries =
                    std::slice::from_raw_parts_mut(cell_map_ref.entries, cell_map_ref.len);
                for entry in entries {
                    // Free the CCellData entries
                    if !entry.key.cell_name.is_null() {
                        let _ = CString::from_raw(entry.key.cell_name);
                    }
                    if !entry.value.cell_name.is_null() {
                        let _ = CString::from_raw(entry.value.cell_name);
                    }
                }
                let _ =
                    Vec::from_raw_parts(cell_map_ref.entries, cell_map_ref.len, cell_map_ref.len);
            }
            let _ = Box::from_raw(cmatch_ref.cell_map);
        }

        // Free the CMatch struct
        let _ = Box::from_raw(cmatch);
    }
}

/// Free a JSON C string returned by serialize functions
#[unsafe(no_mangle)]
pub extern "C" fn free_json_string(json_str: *mut c_char) {
    if !json_str.is_null() {
        unsafe {
            let _ = CString::from_raw(json_str);
        }
    }
}
