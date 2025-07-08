use std::{
    slice,
    path::PathBuf,
    ffi::{CString, CStr},
    os::raw::c_char,
    ptr,
};

pub struct Pattern {
    pub file_loc: PathBuf,
    pub in_ports: Vec<String>,
    pub out_ports: Vec<String>,
    pub inout_ports: Vec<String>,
}

#[repr(C)]
pub struct CPattern {
    file_loc:      *const c_char,

    in_ports:      *const *const c_char,
    in_ports_len:  usize,

    out_ports:     *const *const c_char,
    out_ports_len: usize,

    inout_ports:     *const *const c_char,
    inout_ports_len: usize,
}

#[repr(C)]                 // <-- c_repr must be the first field!
struct CPatternBoxed {
    c_repr: CPattern,

    // Everything below is invisible for C but makes sure the pointers
    // inside `c_repr` stay valid for the life-time of the object.
    file_loc_buf:   CString,

    in_bufs:        Vec<CString>,
    out_bufs:       Vec<CString>,
    inout_bufs:     Vec<CString>,

    in_ptrs:        Vec<*const c_char>,
    out_ptrs:       Vec<*const c_char>,
    inout_ptrs:     Vec<*const c_char>,
}

impl From<Pattern> for CPatternBoxed {
    fn from(p: Pattern) -> Self {
        // Turn path into CString --------------------------------------------
        let file_loc_buf = CString::new(p.file_loc.to_string_lossy().into_owned())
            .expect("Path contained an interior NUL byte");

        // Helper closure to turn Vec<String> --> (Vec<CString>, Vec<*const>)
        fn convert_list(src: Vec<String>) -> (Vec<CString>, Vec<*const c_char>) {
            let bufs: Vec<CString> = src
                .into_iter()
                .map(|s| CString::new(s).expect("String contained NUL"))
                .collect();
            let ptrs: Vec<*const c_char> = bufs.iter().map(|c| c.as_ptr()).collect();
            (bufs, ptrs)
        }

        let (in_bufs,   in_ptrs)   = convert_list(p.in_ports);
        let (out_bufs,  out_ptrs)  = convert_list(p.out_ports);
        let (inout_bufs,inout_ptrs)= convert_list(p.inout_ports);

        // Fill the public C struct ------------------------------------------
        let c_repr = CPattern {
            file_loc: file_loc_buf.as_ptr(),

            in_ports:     in_ptrs.as_ptr(),
            in_ports_len: in_ptrs.len(),

            out_ports:     out_ptrs.as_ptr(),
            out_ports_len: out_ptrs.len(),

            inout_ports:     inout_ptrs.as_ptr(),
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
            let file_loc = PathBuf::from(
                CStr::from_ptr(c.file_loc).to_string_lossy().into_owned()
            );

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
                in_ports:   make_vec(c.in_ports  , c.in_ports_len),
                out_ports:  make_vec(c.out_ports , c.out_ports_len),
                inout_ports:make_vec(c.inout_ports, c.inout_ports_len),
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cpattern_new(
    file_loc:       *const c_char,

    in_ports:       *const *const c_char,
    in_ports_len:   usize,

    out_ports:      *const *const c_char,
    out_ports_len:  usize,

    inout_ports:    *const *const c_char,
    inout_ports_len:usize,
) -> *mut CPattern {
    // Basic parameter validation --------------------------------------------
    if file_loc.is_null() {
        return ptr::null_mut();
    }

    // Build a Pattern from the incoming raw data ----------------------------
    let make_vec =
        |ptr: *const *const c_char, len: usize| -> Vec<String> {
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
        file_loc: PathBuf::from(
            CStr::from_ptr(file_loc).to_string_lossy().into_owned()
        ),
        in_ports:   make_vec(in_ports  , in_ports_len),
        out_ports:  make_vec(out_ports , out_ports_len),
        inout_ports:make_vec(inout_ports, inout_ports_len),
    };

    // Convert to C representation and leak the box so C can hold a pointer --
    let boxed: Box<CPatternBoxed> = Box::new(pattern.into());
    let ptr: *const CPattern = &boxed.c_repr;

    // We turn the same address into a fat pointer for later `free`
    // SAFETY: `c_repr` is the first field, so addresses coincide.
    Box::into_raw(boxed) as *mut CPattern
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cpattern_free(ptr: *mut CPattern) {
    if ptr.is_null() {
        return;
    }
    // Re-cast the address back to the boxed wrapper and drop it.
    let _boxed: Box<CPatternBoxed> = Box::from_raw(ptr as *mut CPatternBoxed);
    // dropping `_boxed` frees everything (CString buffers, Vecs, etc.)
}