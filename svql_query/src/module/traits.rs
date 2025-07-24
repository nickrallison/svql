use crate::ports::{Connection, InPort, OutPort};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::path::PathBuf;
use svql_common::mat::IdString;

pub trait RtlModuleTrait {
    type Result: Debug + RtlModuleResultTrait;

    fn file_path(&self) -> PathBuf;
    fn module_name(&self) -> &'static str;
    fn valid_connections(&self, connections: &HashSet<Connection<InPort, OutPort>>) -> bool {
        todo!()
    }
}

pub trait RtlModuleResultTrait {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self;
}
