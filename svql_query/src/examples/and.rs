use crate::module::lookup;
use crate::module::traits::{RtlModuleResultTrait, RtlModuleTrait};
use crate::ports::{Connection, InPort, OutPort};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use svql_common::mat::IdString;

#[derive(Debug, Clone)]
pub struct And {
    pub a: InPort,
    pub b: InPort,
    pub y: OutPort,
}

impl And {
    pub fn new() -> Self {
        And {
            a: InPort::new("a"),
            b: InPort::new("b"),
            y: OutPort::new("y"),
        }
    }
}

impl RtlModuleTrait for And {
    type Result = AndResult;
    // type ModuleIter = ();

    fn file_path(&self) -> PathBuf {
        "svql_query/verilog/and.v".into()
    }

    fn module_name(&self) -> &'static str {
        "and_gate"
    }

    fn valid_connections(&self, connections: &HashSet<Connection<InPort, OutPort>>) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct AndResult {
    pub a: IdString,
    pub b: IdString,
    pub y: IdString,
}

impl AndResult {
    pub fn new(a: IdString, b: IdString, y: IdString) -> Self {
        AndResult { a, b, y }
    }
}

impl RtlModuleResultTrait for AndResult {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> AndResult {
        AndResult {
            a: lookup(&port_map, "a").expect("Port 'a' not found"),
            b: lookup(&port_map, "b").expect("Port 'b' not found"),
            y: lookup(&port_map, "y").expect("Port 'y' not found"),
        }
    }
}
