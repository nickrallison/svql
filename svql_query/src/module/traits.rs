use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use svql_common::matches::IdString;

pub trait RtlModuleTrait {
    type Result: Debug + RtlModuleResultTrait;
    fn file_path(&self) -> PathBuf;
    fn module_name(&self) -> &'static str;
    // fn valid_connections(&self, connections: &HashSet<Connection<InPort, OutPort>>) -> bool;
    // fn set_instance(&mut self, inst: Arc<String>);
    fn init_full_path(&mut self, full_path: VecDeque<Arc<String>>, height: usize);
}

pub trait RtlModuleResultTrait {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self;
    fn find_port(&self, port_name: VecDeque<Arc<String>>, height: usize) -> Option<&IdString>;
}
