use serde::{Deserialize, Serialize};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    path::PathBuf,
    ptr, slice,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pattern {
    pub file_loc: PathBuf,
    pub in_ports: Vec<String>,
    pub out_ports: Vec<String>,
    pub inout_ports: Vec<String>,
}

#[repr(C)]
pub struct CPattern {
    file_loc: *const c_char,

    in_ports: *const *const c_char,
    in_ports_len: usize,

    out_ports: *const *const c_char,
    out_ports_len: usize,

    inout_ports: *const *const c_char,
    inout_ports_len: usize,
}

#[repr(C)]
struct CPatternBoxed {
    c_repr: CPattern,

    file_loc_buf: CString,

    in_bufs: Vec<CString>,
    out_bufs: Vec<CString>,
    inout_bufs: Vec<CString>,

    in_ptrs: Vec<*const c_char>,
    out_ptrs: Vec<*const c_char>,
    inout_ptrs: Vec<*const c_char>,
}

impl From<Pattern> for CPatternBoxed {
    fn from(p: Pattern) -> Self {
        let file_loc_buf = CString::new(p.file_loc.to_string_lossy().into_owned())
            .expect("Path contained an interior NUL byte");

        fn convert_list(src: Vec<String>) -> (Vec<CString>, Vec<*const c_char>) {
            let bufs: Vec<CString> = src
                .into_iter()
                .map(|s| CString::new(s).expect("String contained NUL"))
                .collect();
            let ptrs: Vec<*const c_char> = bufs.iter().map(|c| c.as_ptr()).collect();
            (bufs, ptrs)
        }

        let (in_bufs, in_ptrs) = convert_list(p.in_ports);
        let (out_bufs, out_ptrs) = convert_list(p.out_ports);
        let (inout_bufs, inout_ptrs) = convert_list(p.inout_ports);

        let c_repr = CPattern {
            file_loc: file_loc_buf.as_ptr(),

            in_ports: in_ptrs.as_ptr(),
            in_ports_len: in_ptrs.len(),

            out_ports: out_ptrs.as_ptr(),
            out_ports_len: out_ptrs.len(),

            inout_ports: inout_ptrs.as_ptr(),
            inout_ports_len: inout_ptrs.len(),
        };

        Self {
            c_repr,
            file_loc_buf,
            in_bufs,
            out_bufs,
            inout_bufs,
            in_ptrs,
            out_ptrs,
            inout_ptrs,
        }
    }
}

impl From<&CPattern> for Pattern {
    fn from(c: &CPattern) -> Self {
        unsafe {
            let file_loc = PathBuf::from(CStr::from_ptr(c.file_loc).to_string_lossy().into_owned());

            let make_vec = |ptr: *const *const c_char, len: usize| -> Vec<String> {
                if ptr.is_null() {
                    Vec::new()
                } else {
                    slice::from_raw_parts(ptr, len)
                        .iter()
                        .map(|&p| CStr::from_ptr(p).to_string_lossy().into_owned())
                        .collect()
                }
            };

            Self {
                file_loc,
                in_ports: make_vec(c.in_ports, c.in_ports_len),
                out_ports: make_vec(c.out_ports, c.out_ports_len),
                inout_ports: make_vec(c.inout_ports, c.inout_ports_len),
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cpattern_new(
    file_loc: *const c_char,

    in_ports: *const *const c_char,
    in_ports_len: usize,

    out_ports: *const *const c_char,
    out_ports_len: usize,

    inout_ports: *const *const c_char,
    inout_ports_len: usize,
) -> *mut CPattern {
    unsafe {
        if file_loc.is_null() {
            return ptr::null_mut();
        }

        let make_vec = |ptr: *const *const c_char, len: usize| -> Vec<String> {
            if ptr.is_null() || len == 0 {
                Vec::new()
            } else {
                slice::from_raw_parts(ptr, len)
                    .iter()
                    .map(|&p| CStr::from_ptr(p).to_string_lossy().into_owned())
                    .collect()
            }
        };

        let pattern = Pattern {
            file_loc: PathBuf::from(CStr::from_ptr(file_loc).to_string_lossy().into_owned()),
            in_ports: make_vec(in_ports, in_ports_len),
            out_ports: make_vec(out_ports, out_ports_len),
            inout_ports: make_vec(inout_ports, inout_ports_len),
        };

        let boxed: Box<CPatternBoxed> = Box::new(pattern.into());
        Box::into_raw(boxed) as *mut CPattern
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cpattern_free(ptr: *mut CPattern) {
    unsafe {
        if ptr.is_null() {
            return;
        }
        let _boxed: Box<CPatternBoxed> = Box::from_raw(ptr as *mut CPatternBoxed);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cpattern_to_json(pattern: *const CPattern) -> *mut c_char {
    unsafe {
        if pattern.is_null() {
            return ptr::null_mut();
        }

        let rust_pattern = Pattern::from(&*pattern);
        match serde_json::to_string_pretty(&rust_pattern) {
            Ok(json_string) => match CString::new(json_string) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => ptr::null_mut(),
            },
            Err(_) => ptr::null_mut(),
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cpattern_from_json(json_str: *const c_char) -> *mut CPattern {
    unsafe {
        if json_str.is_null() {
            return ptr::null_mut();
        }

        let json_cstr = CStr::from_ptr(json_str);
        let json_string = match json_cstr.to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        let pattern: Pattern = match serde_json::from_str(json_string) {
            Ok(p) => p,
            Err(_) => return ptr::null_mut(),
        };

        let boxed: Box<CPatternBoxed> = Box::new(pattern.into());
        Box::into_raw(boxed) as *mut CPattern
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cpattern_json_free(json_str: *mut c_char) {
    unsafe {
        if json_str.is_null() {
            return;
        }
        let _c_string: CString = CString::from_raw(json_str);
    }
}
