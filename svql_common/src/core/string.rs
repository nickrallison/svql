// string.rs

use std::os::raw::c_char;
use std::ffi::{CStr, CString as FfiCString, IntoStringError, NulError};
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::fmt;
use std::ops::Deref;

#[repr(C)]
pub struct CrateCString {
    pub string: std::ffi::CString,
}

// By implementing Deref to CStr, we can call all of CStr's methods directly
// on CrateCString. This removes the need for many helper methods like `as_c_str`.
impl Deref for CrateCString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl CrateCString {
    /// Create a new owned C string. Fails if `s` contains an interior NUL.
    pub fn new(s: &str) -> Result<Self, NulError> {
        let c_string = FfiCString::new(s)?;
        Ok(Self { string: c_string })
    }

    /// Reconstructs from a raw pointer previously returned by `into_raw`.
    /// On drop, the memory will be deallocated.
    ///
    /// # Safety
    /// The pointer must have been previously obtained from `into_raw` and not yet freed.
    pub unsafe fn from_raw(ptr: *mut c_char) -> Self {
        Self { string: unsafe { FfiCString::from_raw(ptr) } }
    }

    /// Borrow as a `&str`. Panics if the C data was not valid UTF-8.
    pub fn as_str(&self) -> &str {
        // self.to_str() is available via Deref<Target=CStr>
        self.to_str().expect("CrateCString should contain valid UTF-8")
    }

    /// Consumes the `CrateCString` and returns the underlying raw pointer.
    /// The caller is responsible for freeing the memory by converting it back
    /// to a `CrateCString` with `from_raw`.
    pub fn into_raw(self) -> *mut c_char {
        self.string.into_raw()
    }

    /// Consume and turn back into a Rust `String`. Returns an error
    /// if the bytes are not valid UTF-8.
    pub fn into_string(self) -> Result<String, IntoStringError> {
        self.string.into_string()
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> usize {
        self.string.as_bytes().len()
    }

}

// Delegate Default implementation
impl Default for CrateCString {
    fn default() -> Self {
        Self { string: FfiCString::default() }
    }
}

// Delegate Clone implementation
impl Clone for CrateCString {
    fn clone(&self) -> Self {
        Self { string: self.string.clone() }
    }
}

// Delegate Debug implementation
impl fmt::Debug for CrateCString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.string.fmt(f)
    }
}

// Delegate PartialEq implementation
impl PartialEq for CrateCString {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}
impl Eq for CrateCString {}

// Delegate PartialOrd implementation
impl PartialOrd for CrateCString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.string.partial_cmp(&other.string)
    }
}

// Delegate Ord implementation
impl Ord for CrateCString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.string.cmp(&other.string)
    }
}

// Delegate Hash implementation
impl Hash for CrateCString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.string.hash(state);
    }
}

impl From<&str> for CrateCString {
    fn from(s: &str) -> Self {
        // The unwrap is safe because we are converting from a valid Rust string slice,
        // which cannot contain interior NUL bytes.
        Self::new(s).unwrap()
    }
}

impl From<String> for CrateCString {
    fn from(s: String) -> Self {
        Self::new(&s).unwrap()
    }
}

impl From<&String> for CrateCString {
    fn from(s: &String) -> Self {
        Self::new(s).unwrap()
    }
}

impl From<CrateCString> for String {
    fn from(cs: CrateCString) -> Self {
        cs.into_string().unwrap()
    }
}

impl From<&CrateCString> for String {
    fn from(cs: &CrateCString) -> Self {
        cs.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::ffi::CStr;

    #[test]
    fn test_new_and_empty() {
        let cs = CrateCString::new("").unwrap();
        // .len() and .is_empty() are now available via Deref
        assert_eq!(cs.len(), 0);
        assert!(cs.is_empty());
        unsafe {
            assert_eq!(CStr::from_ptr(cs.as_ptr()).to_bytes(), b"");
        }
    }

    #[test]
    fn test_default() {
        let cs: CrateCString = Default::default();
        assert_eq!(cs.len(), 0);
        assert!(cs.is_empty());
    }

    #[test]
    fn test_as_str_and_ptr() {
        let cs = CrateCString::new("hello").unwrap();
        assert_eq!(cs.as_str(), "hello");
        assert_eq!(cs.len(), 5);
        // .as_ptr() is available via Deref
        let ptr = cs.as_ptr();
        assert!(!ptr.is_null());
        unsafe {
            assert_eq!(CStr::from_ptr(ptr).to_str().unwrap(), "hello");
        }
    }

    #[test]
    fn test_into_string_and_from_string() {
        let original = "world".to_string();
        let cs: CrateCString = original.clone().into();
        let back: String = cs.clone().into();
        assert_eq!(back, original);
    }

    #[test]
    fn test_new_err_on_nul() {
        assert!(CrateCString::new("a\0b").is_err());
    }

    #[test]
    fn test_clone() {
        let cs1 = CrateCString::new("clone").unwrap();
        let cs2 = cs1.clone();
        assert_eq!(cs1, cs2);
        // They must not share the same pointer
        assert_ne!(cs1.as_ptr(), cs2.as_ptr());
    }

    #[test]
    fn test_drop_no_panic() {
        let cs = CrateCString::new("drop").unwrap();
        drop(cs);
    }

    #[test]
    fn test_debug_format() {
        let cs = CrateCString::new("dbg").unwrap();
        assert_eq!(format!("{:?}", cs), "\"dbg\"");
        let empty: CrateCString = Default::default();
        assert_eq!(format!("{:?}", empty), "\"\"");
    }

    #[test]
    fn test_equality_and_ordering() {
        let a = CrateCString::new("a").unwrap();
        let b = CrateCString::new("b").unwrap();
        let a2 = CrateCString::new("a").unwrap();
        assert_eq!(a, a2);
        assert_ne!(a, b);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_hash() {
        let a = CrateCString::new("a").unwrap();
        let a2 = CrateCString::new("a").unwrap();
        let b = CrateCString::new("b").unwrap();
        let mut set = HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&a2));
        assert!(!set.contains(&b));
        set.insert(b);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_from_raw_roundtrip() {
        let cs1 = CrateCString::new("round").unwrap();
        let raw = cs1.into_raw();
        let cs2 = unsafe { CrateCString::from_raw(raw) };
        assert_eq!(cs2.as_str(), "round");
    }
}