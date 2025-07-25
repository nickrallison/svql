use crate::module::lookup;
use crate::module::traits::{RtlModuleResultTrait, RtlModuleTrait};
use crate::ports::{Connection, InPort, OutPort};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
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

    fn file_path(&self) -> PathBuf {
        "svql_query/verilog/and.v".into()
    }

    fn module_name(&self) -> &'static str {
        "and_gate"
    }

    #[allow(unused_variables)]
    fn valid_connections(&self, connections: &HashSet<Connection<InPort, OutPort>>) -> bool {
        todo!()
    }

    #[allow(unused_variables)]
    fn set_instance(&mut self, inst: Arc<String>) {
        todo!()
    }
    fn init_full_path(&mut self, full_path: VecDeque<Arc<String>>) {
        self.a.init_full_path(full_path.clone());
        self.b.init_full_path(full_path.clone());
        self.y.init_full_path(full_path);
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

    fn find_port(&self, port_name: VecDeque<Arc<String>>) -> Option<&IdString> {
        if port_name.len() != 2 {
            return None;
        }
        if port_name[1] == Arc::new("a".to_string()) {
            Some(&self.a)
        } else if port_name[1] == Arc::new("b".to_string()) {
            Some(&self.b)
        } else if port_name[1] == Arc::new("y".to_string()) {
            Some(&self.y)
        } else {
            None
        }
    }
}
