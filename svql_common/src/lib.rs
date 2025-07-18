
pub mod config;
pub mod core;
pub mod pat;
pub mod mat;

use crate::core::list::List;
use crate::core::string::CrateCString;


#[unsafe(no_mangle)]
pub extern "C" fn print_string(s: &CrateCString) {
    println!("{}", s.as_str());
}

pub type StringList = List<CrateCString>;



#[unsafe(no_mangle)]
pub extern "C" fn string_list_append(list: &mut List<CrateCString>, item: CrateCString) {
    list.append(item);
}