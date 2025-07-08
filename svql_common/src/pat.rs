use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    slice,
    path::PathBuf,
};

#[repr(C)]
pub struct CPattern {
    pub file_loc: *const c_char,
    pub in_ports: *const *const c_char,
    pub out_ports: *const *const c_char,
    pub inout_ports: *const *const c_char,
    pub in_ports_len: usize,
    pub out_ports_len: usize,
    pub inout_ports_len: usize,
}

pub struct Pattern {
    pub file_loc: PathBuf,
    pub in_ports: Vec<String>,
    pub out_ports: Vec<String>,
    pub inout_ports: Vec<String>,
}

impl Pattern {
    pub fn into_cpattern(self) -> CPattern {
        self.into()
    }
}

impl Into<CPattern> for Pattern {
    fn into(self) -> CPattern {
        let file_loc = CString::new(self.file_loc.to_string_lossy().into_owned()).unwrap();
        let in_ports: Vec<*mut c_char> = self.in_ports.iter()
            .map(|s| CString::new(s.clone()).unwrap().into_raw())
            .collect();
        let out_ports: Vec<*mut c_char> = self.out_ports.iter()
            .map(|s| CString::new(s.clone()).unwrap().into_raw())
            .collect();
        let inout_ports: Vec<*mut c_char> = self.inout_ports.iter()
            .map(|s| CString::new(s.clone()).unwrap().into_raw())
            .collect();

        CPattern {
            file_loc: file_loc.into_raw(),
            in_ports: in_ports.as_ptr() as *const *const c_char,
            out_ports: out_ports.as_ptr() as *const *const c_char,
            inout_ports: inout_ports.as_ptr() as *const *const c_char,
            in_ports_len: in_ports.len(),
            out_ports_len: out_ports.len(),
            inout_ports_len: inout_ports.len(),
        }
    }
}

impl CPattern {
    pub fn into_pattern(self) -> Pattern {
        self.into()
    }
}

impl Into<Pattern> for CPattern {
    fn into(self) -> Pattern {
        let file_loc = unsafe { CStr::from_ptr(self.file_loc).to_string_lossy().into_owned() };
        let in_ports = unsafe {
            slice::from_raw_parts(self.in_ports, self.in_ports_len)
                .iter()
                .map(|&s| CStr::from_ptr(s).to_string_lossy().into_owned())
                .collect()
        };
        let out_ports = unsafe {
            slice::from_raw_parts(self.out_ports, self.out_ports_len)
                .iter()
                .map(|&s| CStr::from_ptr(s).to_string_lossy().into_owned())
                .collect()
        };
        let inout_ports = unsafe {
            slice::from_raw_parts(self.inout_ports, self.inout_ports_len)
                .iter()
                .map(|&s| CStr::from_ptr(s).to_string_lossy().into_owned())
                .collect()
        };

        Pattern {
            file_loc: PathBuf::from(file_loc),
            in_ports,
            out_ports,
            inout_ports,
        }
    }
}