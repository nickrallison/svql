// source.rs
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    ffi::{CStr, CString},
    fs,
    io::BufRead,
    os::raw::c_char,
    slice,
};

pub mod source;
pub mod pat;