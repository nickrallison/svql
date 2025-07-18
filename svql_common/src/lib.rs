// pub mod list;
// pub mod mat;
// pub mod pat;
// pub mod source;
pub mod config;
pub mod core;

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