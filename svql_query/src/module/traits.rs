use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use svql_common::id_string::IdString;

use crate::full_path::FullPath;

pub trait RtlModuleTrait {
    type Result: Debug + RtlModuleResultTrait;

    // Basic Functionality
    fn file_path() -> &'static str;
    fn module_name() -> &'static str;
    fn yosys_path() -> &'static str;
    fn svql_pat_plugin_path() -> &'static str;

    // Recursive Functionality
    fn initialize(path: FullPath) -> Self;
}

pub trait RtlModuleResultTrait {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self;
    fn find_port(&self, port_name: VecDeque<Arc<String>>, height: usize) -> Vec<&IdString>;
}
