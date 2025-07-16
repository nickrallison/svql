use std::{
    slice,
    collections::HashMap,
    ffi::{CString, CStr},
    os::raw::c_char,
    ptr,
};
use serde::{Serialize, Deserialize};
use crate::source::{CSourceLoc, SourceLoc};

// /// Represents a match in the source code.
// /// Includes the source location and the ports involved in the match.
// /// Where the keys of each port are the name of the signals specified in the pattern,
// /// and the values are the names of the signals in the source code that match the pattern.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Match {
    pub pattern_keys: Vec<String>,
    pub graph_vals: Vec<String>,
}

// #[repr(C)]
// pub struct CMatch {
//     sources: CSourceLoc,

//     in_ports_len: usize,
//     in_ports_keys: *const *const c_char,
//     in_ports_values: *const *const c_char,
    
//     out_ports_len: usize,
//     out_ports_keys: *const *const c_char,
//     out_ports_values: *const *const c_char,

//     inout_ports_len: usize,
//     inout_ports_keys: *const *const c_char,
//     inout_ports_values: *const *const c_char,
// }

// #[repr(C)]                 // <-- c_repr must be the first field!
// struct CMatchBoxed {
//     c_repr: CMatch,
    
//     sources_c: CSourceLoc,
//     sources_ranges: Vec<crate::source::CSourceRange>,

//     in_keys_bufs: Vec<CString>,
//     in_values_bufs: Vec<CString>,
//     in_keys_ptrs: Vec<*const c_char>,
//     in_values_ptrs: Vec<*const c_char>,

//     out_keys_bufs: Vec<CString>,
//     out_values_bufs: Vec<CString>,
//     out_keys_ptrs: Vec<*const c_char>,
//     out_values_ptrs: Vec<*const c_char>,

//     inout_keys_bufs: Vec<CString>,
//     inout_values_bufs: Vec<CString>,
//     inout_keys_ptrs: Vec<*const c_char>,
//     inout_values_ptrs: Vec<*const c_char>,
// }

// impl From<Match> for CMatchBoxed {
//     fn from(m: Match) -> Self {
//         // Convert source location to C representation
//         let mut sources_ranges: Vec<crate::source::CSourceRange> = Vec::with_capacity(m.sources.ranges.len());
//         for range in m.sources.ranges {
//             let file_cstring = CString::new(range.file).expect("String contained NUL");
//             sources_ranges.push(crate::source::CSourceRange {
//                 file: file_cstring.into_raw(),
//                 line_begin: range.line_begin,
//                 col_begin: range.col_begin,
//                 line_end: range.line_end,
//                 col_end: range.col_end,
//             });
//         }

//         let sources_c = CSourceLoc {
//             ranges: sources_ranges.as_ptr(),
//             len: sources_ranges.len(),
//         };

//         // Helper closure to turn HashMap<String, String> --> (Vec<CString>, Vec<CString>, Vec<*const>, Vec<*const>)
//         fn convert_hashmap(src: HashMap<String, String>) -> (Vec<CString>, Vec<CString>, Vec<*const c_char>, Vec<*const c_char>) {
//             let keys_bufs: Vec<CString> = src
//                 .keys()
//                 .map(|s| CString::new(s.clone()).expect("String contained NUL"))
//                 .collect();
//             let values_bufs: Vec<CString> = src
//                 .values()
//                 .map(|s| CString::new(s.clone()).expect("String contained NUL"))
//                 .collect();
//             let keys_ptrs: Vec<*const c_char> = keys_bufs.iter().map(|c| c.as_ptr()).collect();
//             let values_ptrs: Vec<*const c_char> = values_bufs.iter().map(|c| c.as_ptr()).collect();
//             (keys_bufs, values_bufs, keys_ptrs, values_ptrs)
//         }

//         let (in_keys_bufs, in_values_bufs, in_keys_ptrs, in_values_ptrs) = convert_hashmap(m.in_ports);
//         let (out_keys_bufs, out_values_bufs, out_keys_ptrs, out_values_ptrs) = convert_hashmap(m.out_ports);
//         let (inout_keys_bufs, inout_values_bufs, inout_keys_ptrs, inout_values_ptrs) = convert_hashmap(m.inout_ports);

//         // Fill the public C struct
//         let c_repr = CMatch {
//             sources: sources_c,

//             in_ports_len: in_keys_ptrs.len(),
//             in_ports_keys: in_keys_ptrs.as_ptr(),
//             in_ports_values: in_values_ptrs.as_ptr(),

//             out_ports_len: out_keys_ptrs.len(),
//             out_ports_keys: out_keys_ptrs.as_ptr(),
//             out_ports_values: out_values_ptrs.as_ptr(),

//             inout_ports_len: inout_keys_ptrs.len(),
//             inout_ports_keys: inout_keys_ptrs.as_ptr(),
//             inout_ports_values: inout_values_ptrs.as_ptr(),
//         };

//         Self {
//             c_repr,
//             sources_c: CSourceLoc {
//                 ranges: sources_ranges.as_ptr(),
//                 len: sources_ranges.len(),
//             },
//             sources_ranges,
//             in_keys_bufs,
//             in_values_bufs,
//             in_keys_ptrs,
//             in_values_ptrs,
//             out_keys_bufs,
//             out_values_bufs,
//             out_keys_ptrs,
//             out_values_ptrs,
//             inout_keys_bufs,
//             inout_values_bufs,
//             inout_keys_ptrs,
//             inout_values_ptrs,
//         }
//     }
// }

// impl From<&CMatch> for Match {
//     fn from(c: &CMatch) -> Self {
//         unsafe {
//             // Convert CSourceLoc to SourceLoc
//             let sources = if c.sources.ranges.is_null() || c.sources.len == 0 {
//                 SourceLoc { ranges: Vec::new() }
//             } else {
//                 let ranges_slice = slice::from_raw_parts(c.sources.ranges, c.sources.len);
//                 let mut ranges = Vec::with_capacity(c.sources.len);
//                 for cr in ranges_slice {
//                     if !cr.file.is_null() {
//                         let file = CStr::from_ptr(cr.file).to_string_lossy().into_owned();
//                         ranges.push(crate::source::SourceRange {
//                             file,
//                             line_begin: cr.line_begin,
//                             col_begin: cr.col_begin,
//                             line_end: cr.line_end,
//                             col_end: cr.col_end,
//                         });
//                     }
//                 }
//                 SourceLoc { ranges }
//             };

//             let make_hashmap = |keys_ptr: *const *const c_char, values_ptr: *const *const c_char, len: usize| -> HashMap<String, String> {
//                 if keys_ptr.is_null() || values_ptr.is_null() {
//                     HashMap::new()
//                 } else {
//                     let keys_slice = slice::from_raw_parts(keys_ptr, len);
//                     let values_slice = slice::from_raw_parts(values_ptr, len);
//                     keys_slice
//                         .iter()
//                         .zip(values_slice.iter())
//                         .map(|(&k_ptr, &v_ptr)| {
//                             let key = CStr::from_ptr(k_ptr).to_string_lossy().into_owned();
//                             let value = CStr::from_ptr(v_ptr).to_string_lossy().into_owned();
//                             (key, value)
//                         })
//                         .collect()
//                 }
//             };

//             Self {
//                 sources,
//                 in_ports: make_hashmap(c.in_ports_keys, c.in_ports_values, c.in_ports_len),
//                 out_ports: make_hashmap(c.out_ports_keys, c.out_ports_values, c.out_ports_len),
//                 inout_ports: make_hashmap(c.inout_ports_keys, c.inout_ports_values, c.inout_ports_len),
//             }
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn cmatch_new(
//     sources: *const CSourceLoc,

//     in_ports_keys: *const *const c_char,
//     in_ports_values: *const *const c_char,
//     in_ports_len: usize,

//     out_ports_keys: *const *const c_char,
//     out_ports_values: *const *const c_char,
//     out_ports_len: usize,

//     inout_ports_keys: *const *const c_char,
//     inout_ports_values: *const *const c_char,
//     inout_ports_len: usize,
// ) -> *mut CMatch {
//     // Basic parameter validation
//     if sources.is_null() {
//         return ptr::null_mut();
//     }

//     // Build a Match from the incoming raw data
//     let make_hashmap = |keys_ptr: *const *const c_char, values_ptr: *const *const c_char, len: usize| -> HashMap<String, String> {
//         if keys_ptr.is_null() || values_ptr.is_null() || len == 0 {
//             HashMap::new()
//         } else {
//             let keys_slice = slice::from_raw_parts(keys_ptr, len);
//             let values_slice = slice::from_raw_parts(values_ptr, len);
//             keys_slice
//                 .iter()
//                 .zip(values_slice.iter())
//                 .map(|(&k_ptr, &v_ptr)| {
//                     let key = CStr::from_ptr(k_ptr).to_string_lossy().into_owned();
//                     let value = CStr::from_ptr(v_ptr).to_string_lossy().into_owned();
//                     (key, value)
//                 })
//                 .collect()
//         }
//     };

//     let cmatch: CMatch = CMatch {
//         sources: *sources,

//         in_ports_len,
//         in_ports_keys,
//         in_ports_values,

//         out_ports_len,
//         out_ports_keys,
//         out_ports_values,

//         inout_ports_len,
//         inout_ports_keys,
//         inout_ports_values,
//     };



//     // Convert to C representation and leak the box so C can hold a pointer
//     let boxed_cmatch: Box<CMatch> = Box::new(cmatch);

//     // SAFETY: `c_repr` is the first field, so addresses coincide.
//     Box::into_raw(boxed_cmatch) as *mut CMatch
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn cmatch_free(ptr: *mut CMatch) {
//     if ptr.is_null() {
//         return;
//     }
//     // Re-cast the address back to the boxed wrapper and drop it.
//     let _boxed: Box<CMatchBoxed> = Box::from_raw(ptr as *mut CMatchBoxed);
//     // dropping `_boxed` frees everything (CString buffers, Vecs, etc.)
// }

// /// Serialize a CMatch to JSON string
// /// Returns a pointer to a C string that must be freed with cmatch_json_free
// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn cmatch_to_json(mat: *const CMatch) -> *mut c_char {
//     if mat.is_null() {
//         return ptr::null_mut();
//     }

//     let rust_match = Match::from(&*mat);
//     match serde_json::to_string_pretty(&rust_match) {
//         Ok(json_string) => {
//             match CString::new(json_string) {
//                 Ok(c_string) => c_string.into_raw(),
//                 Err(_) => ptr::null_mut(),
//             }
//         }
//         Err(_) => ptr::null_mut(),
//     }
// }

// /// Deserialize a JSON string to CMatch
// /// Takes ownership of the JSON string and returns a CMatch pointer
// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn cmatch_from_json(json_str: *const c_char) -> *mut CMatch {
//     if json_str.is_null() {
//         return ptr::null_mut();
//     }

//     let json_cstr = CStr::from_ptr(json_str);
//     let json_string = match json_cstr.to_str() {
//         Ok(s) => s,
//         Err(_) => return ptr::null_mut(),
//     };

//     let mat: Match = match serde_json::from_str(json_string) {
//         Ok(m) => m,
//         Err(_) => return ptr::null_mut(),
//     };

//     // Convert to C representation and leak the box so C can hold a pointer
//     let boxed: Box<CMatchBoxed> = Box::new(mat.into());
//     Box::into_raw(boxed) as *mut CMatch
// }

// /// Free a JSON string returned by cmatch_to_json
// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn cmatch_json_free(json_str: *mut c_char) {
//     if json_str.is_null() {
//         return;
//     }
//     let _c_string = CString::from_raw(json_str);
//     // CString will be dropped and memory freed
// }

// impl Drop for CMatchBoxed {
//     fn drop(&mut self) {
//         // Free the file strings from sources_ranges
//         for range in &self.sources_ranges {
//             if !range.file.is_null() {
//                 unsafe {
//                     let _ = CString::from_raw(range.file as *mut c_char);
//                 }
//             }
//         }
//     }
// }

