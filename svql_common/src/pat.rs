// pattern.rs

use serde::{Serialize, Deserialize};
use std::path::{PathBuf};
use crate::core::string::CrateCString;
use crate::core::list::List;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub file_loc: PathBuf,
    pub in_ports: Vec<String>,
    pub out_ports: Vec<String>,
    pub inout_ports: Vec<String>,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CPattern {
    pub file_loc: CrateCString,
    pub in_ports: List<CrateCString>,
    pub out_ports: List<CrateCString>,
    pub inout_ports: List<CrateCString>,
}

impl From<&Pattern> for CPattern {
    fn from(p: &Pattern) -> Self {
        CPattern {
            file_loc: CrateCString::from(p.file_loc.to_string_lossy().as_ref()),
            in_ports: p.in_ports.iter().map(|s| CrateCString::from(s.as_str())).collect(),
            out_ports: p.out_ports.iter().map(|s| CrateCString::from(s.as_str())).collect(),
            inout_ports: p.inout_ports.iter().map(|s| CrateCString::from(s.as_str())).collect(),
        }
    }
}

impl From<&CPattern> for Pattern {
    fn from(c: &CPattern) -> Self {
        Pattern {
            file_loc: PathBuf::from(c.file_loc.as_str()),
            in_ports: c.in_ports.as_slice().iter().map(|s| s.as_str().to_string()).collect(),
            out_ports: c.out_ports.as_slice().iter().map(|s| s.as_str().to_string()).collect(),
            inout_ports: c.inout_ports.as_slice().iter().map(|s| s.as_str().to_string()).collect(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pattern_new() -> *mut CPattern {
    let p = Pattern {
        file_loc: PathBuf::new(),
        in_ports: Vec::new(),
        out_ports: Vec::new(),
        inout_ports: Vec::new(),
    };
    Box::into_raw(Box::new(CPattern::from(&p)))
}

#[unsafe(no_mangle)]
pub extern "C" fn pattern_clone(p: *mut CPattern) -> *mut CPattern {
    if p.is_null() {
        return std::ptr::null_mut();
    }
    let c_pattern_ref = unsafe { &*p };
    let r: CPattern = c_pattern_ref.clone();
    Box::into_raw(Box::new(r))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pattern_destroy(p: *mut CPattern) {
    if !p.is_null() {
        unsafe { let _ = Box::from_raw(p); };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn pattern_eq(a: &CPattern, b: &CPattern) -> bool {
    a == b
}

#[unsafe(no_mangle)]
pub extern "C" fn pattern_debug_string(p: &CPattern) -> CrateCString {
    let rust_p = Pattern::from(p);
    let s = format!("{:?}", rust_p);
    CrateCString::from(s.as_str())
}

#[unsafe(no_mangle)]
pub extern "C" fn pattern_to_json(p: &CPattern) -> CrateCString {
    let rust_p = Pattern::from(p);
    match serde_json::to_string(&rust_p) {
        Ok(json) => CrateCString::from(json.as_str()),
        Err(e) => panic!("Failed to serialize to JSON: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn make_sample_pattern() -> Pattern {
        Pattern {
            file_loc: PathBuf::from("/tmp/foo.sv"),
            in_ports: vec!["a".to_string(), "b".to_string()],
            out_ports: vec!["c".to_string()],
            inout_ports: vec!["d".to_string(), "e".to_string()],
        }
    }

    #[test]
    fn test_roundtrip_conversion() {
        let orig = make_sample_pattern();
        let c = CPattern::from(&orig);
        let back = Pattern::from(&c);
        assert_eq!(orig, back);
    }

    #[test]
    fn test_clone_and_eq() {
        let c1 = CPattern::from(&make_sample_pattern());
        let c2 = c1.clone();
        assert_eq!(c1, c2);
        assert_ne!(&c1 as *const _, &c2 as *const _);
    }

    #[test]
    fn test_hash() {
        let c1 = CPattern::from(&make_sample_pattern());
        let c2 = c1.clone();
        let mut set = HashSet::new();
        set.insert(c1);
        assert!(set.contains(&c2));
    }

    #[test]
    fn test_debug_string() {
        let c = CPattern::from(&make_sample_pattern());
        let dbg = pattern_debug_string(&c);
        let s = dbg.as_str();
        assert!(s.contains("file_loc"));
        drop(dbg);
    }

    #[test]
    fn test_ffi_lifecycle() {
        unsafe {
            let c = pattern_new();
            let c2 = pattern_clone(c);
            assert!(pattern_eq(&*c, &*c2));
            let _ = pattern_debug_string(&*c);
        
            pattern_destroy(c2);
            pattern_destroy(c);
        }
    }
}