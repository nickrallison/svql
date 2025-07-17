use std::os::raw::c_char;

#[repr(C)]
pub struct CString {
    pub data: *mut c_char,
}

impl From<String> for CString {
    fn from(s: String) -> Self {
        let c_string = std::ffi::CString::new(s).expect("Failed to create CString");
        CString {
            data: c_string.into_raw(),
        }
    }
}

impl Into<String> for CString {
    fn into(self) -> String {
        if self.data.is_null() {
            return String::new();
        }
        unsafe {
            let c_str = std::ffi::CStr::from_ptr(self.data);
            c_str.to_string_lossy().into_owned()
        }
    }
}

impl Drop for CString {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                let _ = std::ffi::CString::from_raw(self.data);
            }
        }
    }
}