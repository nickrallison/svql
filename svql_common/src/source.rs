use lazy_static::lazy_static;
use regex::Regex;
use std::{
    ffi::{CStr, CString},
    fs,
    io::BufRead,
    os::raw::c_char,
    slice,
};

//
// FFI‐Safe Structs
//

#[repr(C)]
pub struct CSourceRange {
    pub file: *const c_char,
    pub line_begin: usize,
    pub col_begin: usize,
    pub line_end: usize,
    pub col_end: usize,
}

#[repr(C)]
pub struct CSourceLoc {
    pub ranges: *const CSourceRange,
    pub len: usize,
}

//
// Native Rust Types
//

pub struct SourceRange {
    pub file: String,
    pub line_begin: usize,
    pub col_begin: usize,
    pub line_end: usize,
    pub col_end: usize,
}

pub struct SourceLoc {
    pub ranges: Vec<SourceRange>,
}

//
// Regex for parsing SourceRange strings
//

lazy_static! {
    static ref SRANGE_RE: Regex = Regex::new(r"^(.*):(\d+)\.(\d+)-(\d+)\.(\d+)$").unwrap();
}

//
// Impl SourceRange
//

impl SourceRange {
    pub fn to_string(&self) -> String {
        format!(
            "{}:{}.{}-{}.{}",
            self.file, self.line_begin, self.col_begin, self.line_end, self.col_end
        )
    }

    pub fn to_string_pretty(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.to_string());
        out.push('\n');

        // read file
        let content = match fs::read_to_string(&self.file) {
            Ok(s) => s,
            Err(_) => {
                out.push_str(&format!("(cannot open \"{}\")\n", self.file));
                return out;
            }
        };

        // split into lines, strip trailing '\r'
        let lines: Vec<String> = content
            .lines()
            .map(|l| {
                let mut l = l.to_string();
                if l.ends_with('\r') {
                    l.pop();
                }
                l
            })
            .collect();

        // bounds check
        if self.line_begin == 0 || self.line_begin > lines.len() {
            out.push_str(&format!("(file has only {} lines)\n", lines.len()));
            return out;
        }

        let src = &lines[self.line_begin - 1];
        let lb_str = self.line_begin.to_string();
        out.push_str(&format!("{} | {}\n", lb_str, src));

        // build marker
        let mut marker = String::new();
        marker.push_str(&" ".repeat(lb_str.len()));
        marker.push_str(" | ");
        for i in 1..self.col_begin {
            if let Some(ch) = src.chars().nth(i - 1) {
                if ch == '\t' {
                    marker.push('\t');
                } else {
                    marker.push(' ');
                }
            } else {
                marker.push(' ');
            }
        }
        let mut width = if self.line_begin == self.line_end {
            self.col_end
                .saturating_sub(self.col_begin)
                .saturating_add(1)
        } else {
            1
        };
        if width == 0 {
            width = 1;
        }
        marker.push_str(&"^".repeat(width));
        out.push_str(&marker);
        out.push('\n');

        out
    }

    pub fn parse(s: &str) -> Option<SourceRange> {
        let caps = SRANGE_RE.captures(s)?;
        Some(SourceRange {
            file: caps.get(1)?.as_str().to_string(),
            line_begin: caps.get(2)?.as_str().parse().ok()?,
            col_begin: caps.get(3)?.as_str().parse().ok()?,
            line_end: caps.get(4)?.as_str().parse().ok()?,
            col_end: caps.get(5)?.as_str().parse().ok()?,
        })
    }
}

//
// Impl SourceLoc
//

impl SourceLoc {
    pub fn empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn append(&mut self, r: SourceRange) {
        self.ranges.push(r);
    }

    pub fn to_string(&self, sep: char) -> String {
        let mut out = String::new();
        for (i, r) in self.ranges.iter().enumerate() {
            if i > 0 {
                out.push(sep);
            }
            out.push_str(&r.to_string());
        }
        out
    }

    pub fn to_string_pretty(&self) -> String {
        let mut out = String::new();
        for (i, r) in self.ranges.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&r.to_string_pretty());
        }
        out
    }

    pub fn parse(s: &str, sep: char) -> Option<SourceLoc> {
        let mut ranges = Vec::new();
        for token in s.split(sep).filter(|t| !t.is_empty()) {
            let r = SourceRange::parse(token)?;
            ranges.push(r);
        }
        Some(SourceLoc { ranges })
    }
}

//
// FFI Helpers
//

/// Free a string previously returned by any of the `*_to_string*` functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Parse a single SourceRange from a C string.
/// Returns null on bad input.
/// Caller owns the returned CSourceRange* and must call `svql_source_range_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_range_parse(s: *const c_char) -> *mut CSourceRange {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let s = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let sr = match SourceRange::parse(s) {
        Some(sr) => sr,
        None => return std::ptr::null_mut(),
    };
    let cs = CString::new(sr.file).unwrap();
    let file_ptr = cs.into_raw();
    let cr = CSourceRange {
        file: file_ptr,
        line_begin: sr.line_begin,
        col_begin: sr.col_begin,
        line_end: sr.line_end,
        col_end: sr.col_end,
    };
    Box::into_raw(Box::new(cr))
}

/// Turn a CSourceRange into its compact string form.  
/// Returns a malloc’d C string (must be freed by `svql_free_string`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_range_to_string(r: *const CSourceRange) -> *mut c_char {
    if r.is_null() {
        return std::ptr::null_mut();
    }
    let r = &*r;
    if r.file.is_null() {
        return std::ptr::null_mut();
    }
    let file = CStr::from_ptr(r.file).to_string_lossy().into_owned();
    let sr = SourceRange {
        file,
        line_begin: r.line_begin,
        col_begin: r.col_begin,
        line_end: r.line_end,
        col_end: r.col_end,
    };
    CString::new(sr.to_string()).unwrap().into_raw()
}

/// Turn a CSourceRange into its “pretty” multi‐line form.  
/// Returns a malloc’d C string (must be freed by `svql_free_string`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_range_to_string_pretty(r: *const CSourceRange) -> *mut c_char {
    if r.is_null() {
        return std::ptr::null_mut();
    }
    let r = &*r;
    if r.file.is_null() {
        return std::ptr::null_mut();
    }
    let file = CStr::from_ptr(r.file).to_string_lossy().into_owned();
    let sr = SourceRange {
        file,
        line_begin: r.line_begin,
        col_begin: r.col_begin,
        line_end: r.line_end,
        col_end: r.col_end,
    };
    CString::new(sr.to_string_pretty()).unwrap().into_raw()
}

/// Free a CSourceRange previously returned by `svql_source_range_parse`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_range_free(ptr: *mut CSourceRange) {
    if ptr.is_null() {
        return;
    }
    let cr = Box::from_raw(ptr);
    if !cr.file.is_null() {
        let _ = CString::from_raw(cr.file as *mut c_char);
    }
    // Box drops here
}

/// Parse a SourceLoc (a list of ranges) from a C‐string, using separator `sep`.  
/// Returns null on error. Caller must free with `svql_source_loc_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_loc_parse(s: *const c_char, sep: c_char) -> *mut CSourceLoc {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let s = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let sep = sep as u8 as char;
    let sl = match SourceLoc::parse(s, sep) {
        Some(sl) => sl,
        None => return std::ptr::null_mut(),
    };

    // deep‐copy into a Vec<CSourceRange>
    let mut v: Vec<CSourceRange> = Vec::with_capacity(sl.ranges.len());
    for r in sl.ranges {
        let cs = CString::new(r.file).unwrap();
        v.push(CSourceRange {
            file: cs.into_raw(),
            line_begin: r.line_begin,
            col_begin: r.col_begin,
            line_end: r.line_end,
            col_end: r.col_end,
        });
    }
    let len = v.len();
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);

    let cl = CSourceLoc { ranges: ptr, len };
    Box::into_raw(Box::new(cl))
}

/// Turn a CSourceLoc into its compact string form (using `sep`).  
/// Returns malloc’d C string, free with `svql_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_loc_to_string(
    loc: *const CSourceLoc,
    sep: c_char,
) -> *mut c_char {
    if loc.is_null() {
        return std::ptr::null_mut();
    }
    let cl = &*loc;
    let sep = sep as u8 as char;
    let mut ranges = Vec::with_capacity(cl.len);
    if !cl.ranges.is_null() {
        let slice = slice::from_raw_parts(cl.ranges, cl.len);
        for cr in slice {
            if cr.file.is_null() {
                continue;
            }
            let file = CStr::from_ptr(cr.file).to_string_lossy().into_owned();
            ranges.push(SourceRange {
                file,
                line_begin: cr.line_begin,
                col_begin: cr.col_begin,
                line_end: cr.line_end,
                col_end: cr.col_end,
            });
        }
    }
    let sl = SourceLoc { ranges };
    CString::new(sl.to_string(sep)).unwrap().into_raw()
}

/// Turn a CSourceLoc into its “pretty” multi‐line form.  
/// Returns malloc’d C string, free with `svql_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_loc_to_string_pretty(loc: *const CSourceLoc) -> *mut c_char {
    if loc.is_null() {
        return std::ptr::null_mut();
    }
    let cl = &*loc;
    let mut ranges = Vec::with_capacity(cl.len);
    if !cl.ranges.is_null() {
        let slice = slice::from_raw_parts(cl.ranges, cl.len);
        for cr in slice {
            if cr.file.is_null() {
                continue;
            }
            let file = CStr::from_ptr(cr.file).to_string_lossy().into_owned();
            ranges.push(SourceRange {
                file,
                line_begin: cr.line_begin,
                col_begin: cr.col_begin,
                line_end: cr.line_end,
                col_end: cr.col_end,
            });
        }
    }
    let sl = SourceLoc { ranges };
    CString::new(sl.to_string_pretty()).unwrap().into_raw()
}

/// Free a CSourceLoc and all of its inner allocations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_loc_free(ptr: *mut CSourceLoc) {
    if ptr.is_null() {
        return;
    }
    let cl = Box::from_raw(ptr);
    let arr = cl.ranges as *mut CSourceRange;
    let len = cl.len;
    if !arr.is_null() {
        // free each file‐string
        for cr in slice::from_raw_parts(arr, len) {
            if !cr.file.is_null() {
                let _ = CString::from_raw(cr.file as *mut c_char);
            }
        }
        // free the array itself
        let _ = Vec::from_raw_parts(arr, len, len);
    }
    // cl drops here
}

/// Is this SourceLoc empty?
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_loc_empty(loc: *const CSourceLoc) -> bool {
    if loc.is_null() {
        true
    } else {
        (&*loc).len == 0
    }
}

/// Append one CSourceRange to a CSourceLoc (deep‐copies all strings).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn svql_source_loc_append(loc: *mut CSourceLoc, range: *const CSourceRange) {
    if loc.is_null() || range.is_null() {
        return;
    }
    let cl = &mut *loc;
    let old_ptr = cl.ranges as *mut CSourceRange;
    let old_len = cl.len;

    // build a new Vec<CSourceRange> with capacity+1
    let mut nv: Vec<CSourceRange> = Vec::with_capacity(old_len + 1);

    // copy old entries
    if !old_ptr.is_null() {
        let slice = slice::from_raw_parts(old_ptr, old_len);
        for cr in slice {
            if cr.file.is_null() {
                continue;
            }
            let bytes = CStr::from_ptr(cr.file).to_bytes();
            let cs = CString::new(bytes).unwrap();
            nv.push(CSourceRange {
                file: cs.into_raw(),
                line_begin: cr.line_begin,
                col_begin: cr.col_begin,
                line_end: cr.line_end,
                col_end: cr.col_end,
            });
        }
        // drop old
        for cr in slice {
            if !cr.file.is_null() {
                let _ = CString::from_raw(cr.file as *mut c_char);
            }
        }
        let _ = Vec::from_raw_parts(old_ptr, old_len, old_len);
    }

    // copy the new one
    let cr = &*range;
    if !cr.file.is_null() {
        let bytes = CStr::from_ptr(cr.file).to_bytes();
        let cs = CString::new(bytes).unwrap();
        nv.push(CSourceRange {
            file: cs.into_raw(),
            line_begin: cr.line_begin,
            col_begin: cr.col_begin,
            line_end: cr.line_end,
            col_end: cr.col_end,
        });
    }

    // leak the new Vec
    let ptr = nv.as_mut_ptr();
    let len = nv.len();
    std::mem::forget(nv);

    cl.ranges = ptr;
    cl.len = len;
}
