
pub mod config;
pub mod core;
pub mod pat;

use crate::core::list::List;
use crate::core::string::CrateCString;


#[unsafe(no_mangle)]
pub extern "C" fn print_string(s: &CrateCString) {
    println!("{}", s.as_str());
}

pub type StringList = List<CrateCString>;

#[unsafe(no_mangle)]
pub extern "C" fn print_string_list(list: &StringList) {
    for item in list.iter() {
        println!("{}", item.as_str());
    }
}