// string.rs

use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::ptr;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[repr(C)]
pub struct CrateCString {
    pub string: *mut c_char,
}

impl CrateCString {
    pub fn as_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.string) }
    }
    pub fn as_str(&self) -> &str {
        self.as_cstr().to_str().unwrap()
    }
    pub fn len(&self) -> usize {
        self.as_cstr().to_bytes().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for CrateCString {
    fn drop(&mut self) {
        if !self.string.is_null() {
            unsafe {
                // Reconstruct CString to free memory
                let _ = CString::from_raw(self.string);
            }
            self.string = ptr::null_mut();
        }
    }
}

impl Clone for CrateCString {
    fn clone(&self) -> Self {
        let cstr = self.as_cstr();
        let cloned = CString::new(cstr.to_bytes()).unwrap();
        CrateCString {
            string: cloned.into_raw(),
        }
    }
}

impl std::fmt::Debug for CrateCString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_cstr().to_str() {
            Ok(s) => write!(f, "CrateCString({:?})", s),
            Err(_) => write!(f, "CrateCString(<invalid utf8>)"),
        }
    }
}

impl PartialEq for CrateCString {
    fn eq(&self, other: &Self) -> bool {
        self.as_cstr().to_bytes() == other.as_cstr().to_bytes()
    }
}
impl Eq for CrateCString {}

impl PartialOrd for CrateCString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_cstr().to_bytes().partial_cmp(other.as_cstr().to_bytes())
    }
}
impl Ord for CrateCString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_cstr().to_bytes().cmp(other.as_cstr().to_bytes())
    }
}

impl Hash for CrateCString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_cstr().to_bytes().hash(state)
    }
}

impl Default for CrateCString {
    fn default() -> Self {
        let empty = CString::new("").unwrap();
        CrateCString {
            string: empty.into_raw(),
        }
    }
}

impl From<&str> for CrateCString {
    fn from(s: &str) -> Self {
        let cstr = CString::new(s).unwrap();
        CrateCString {
            string: cstr.into_raw(),
        }
    }
}
impl From<String> for CrateCString {
    fn from(s: String) -> Self {
        CrateCString::from(s.as_str())
    }
}
impl From<CString> for CrateCString {
    fn from(cstr: CString) -> Self {
        CrateCString {
            string: cstr.into_raw(),
        }
    }
}
impl From<&CStr> for CrateCString {
    fn from(cstr: &CStr) -> Self {
        let cstr = CString::new(cstr.to_bytes()).unwrap();
        CrateCString {
            string: cstr.into_raw(),
        }
    }
}
impl From<CrateCString> for CString {
    fn from(mut s: CrateCString) -> Self {
        if s.string.is_null() {
            CString::new("").unwrap()
        } else {
            unsafe { CString::from_raw(std::mem::replace(&mut s.string, ptr::null_mut())) }
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn crate_cstring_new(s: *const c_char) -> CrateCString {
    if s.is_null() {
        CrateCString::default()
    } else {
        let cstr = unsafe { CStr::from_ptr(s) };
        CrateCString::from(cstr)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn crate_cstring_clone(s: &CrateCString) -> CrateCString {
    s.clone()
}

#[unsafe(no_mangle)]
pub extern "C" fn crate_cstring_destroy(s: *mut CrateCString) {
    if !s.is_null() {
        unsafe { let _ = Box::from_raw(s); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn crate_cstring_len(s: &CrateCString) -> usize {
    s.len()
}

#[unsafe(no_mangle)]
pub extern "C" fn crate_cstring_eq(a: &CrateCString, b: &CrateCString) -> bool {
    a == b
}

#[unsafe(no_mangle)]
pub extern "C" fn crate_cstring_cmp(a: &CrateCString, b: &CrateCString) -> i32 {
    use std::cmp::Ordering::*;
    match a.cmp(b) {
        Less => -1,
        Equal => 0,
        Greater => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::collections::HashSet;

    #[test]
    fn test_new_and_default() {
        let s = CrateCString::from("hello");
        assert_eq!(s.as_str(), "hello");
        let d = CrateCString::default();
        assert_eq!(d.as_str(), "");
    }

    #[test]
    fn test_clone() {
        let s1 = CrateCString::from("abc");
        let s2 = s1.clone();
        assert_eq!(s1, s2);
        assert_ne!(s1.string, s2.string); // different pointers
    }

    #[test]
    fn test_eq_ord() {
        let a = CrateCString::from("a");
        let b = CrateCString::from("b");
        let a2 = CrateCString::from("a");
        assert_eq!(a, a2);
        assert_ne!(a, b);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_hash() {
        let a = CrateCString::from("foo");
        let b = CrateCString::from("foo");
        let c = CrateCString::from("bar");
        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
        assert!(!set.contains(&c));
    }

    #[test]
    fn test_len_is_empty() {
        let s = CrateCString::from("abc");
        assert_eq!(s.len(), 3);
        assert!(!s.is_empty());
        let empty = CrateCString::default();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_from_cstring() {
        let orig = CString::new("xyz").unwrap();
        let s = CrateCString::from(orig.clone());
        assert_eq!(s.as_str(), "xyz");
        let back: CString = s.clone().into();
        assert_eq!(back, orig);
    }

    #[test]
    fn test_drop() {
        let s = CrateCString::from("dropme");
        drop(s);
        // miri will check for double free, etc.
    }

    #[test]
    fn test_ffi_functions() {
        let orig = CString::new("ffi").unwrap();
        let s = crate_cstring_new(orig.as_ptr());
        assert_eq!(s.as_str(), "ffi");
        let len = crate_cstring_len(&s);
        assert_eq!(len, 3);
        let s2 = crate_cstring_clone(&s);
        assert!(crate_cstring_eq(&s, &s2));
        assert_eq!(crate_cstring_cmp(&s, &s2), 0);
        crate_cstring_destroy(Box::into_raw(Box::new(s2)));
        crate_cstring_destroy(Box::into_raw(Box::new(s)));
    }
}
