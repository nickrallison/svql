use crate::full_path::FullPath;
use crate::module::lookup;
use crate::module::traits::{RtlModuleResultTrait, RtlModuleTrait};
use crate::ports::{InPort, OutPort};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use svql_common::id_string::IdString;
use svql_query_proc_macro::module;

// #[module(
//     file = "examples/patterns/basic/and/verilog/and.v",
//     module = "and_gate",
//     yosys = "./yosys/yosys",
//     svql_pat_plugin_path = "./build/svql_pat_lib/libsvql_pat_lib.so"
// )]
// pub struct And;

// ########################

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct And {
    pub a: InPort,
    pub b: InPort,
    pub y: OutPort,
}
impl RtlModuleTrait for And {
    type Result = AndResult;

    // Basic Functionality
    fn file_path() -> &'static str {
        "examples/patterns/basic/and/verilog/and.v"
    }
    fn module_name() -> &'static str {
        "and_gate"
    }
    fn yosys_path() -> &'static str {
        "./yosys/yosys"
    }
    fn svql_pat_plugin_path() -> &'static str {
        "./build/svql_pat_lib/libsvql_pat_lib.so"
    }   

    fn initialize(path: FullPath) -> Self {
        Self {
            a: InPort::new("a", path.clone()),
            b: InPort::new("b", path.clone()),
            y: OutPort::new("y", path.clone()),
        }
    }

    fn root() -> Self {
        let path = FullPath::root(Self::module_name());
        Self {
            a: InPort::new("a", path.clone()),
            b: InPort::new("b", path.clone()),
            y: OutPort::new("y", path.clone()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndResult {
    pub a: IdString,
    pub b: IdString,
    pub y: IdString,
}

impl RtlModuleResultTrait for AndResult {
    fn from_portmap(
        port_map: HashMap<IdString, IdString>,
    ) -> Self {
        Self {
            a: crate::module::lookup(&port_map, "a").expect("Port \'a\' not found"),
            b: crate::module::lookup(&port_map, "b").expect("Port \'b\' not found"),
            y: crate::module::lookup(&port_map, "y").expect("Port \'y\' not found"),
        }
    }
    fn find_port(
        &self,
        port_name: VecDeque<Arc<String>>,
        height: usize,
    ) -> Vec<&svql_common::id_string::IdString> {
        let port_name = port_name.get(height + 1).map(|s| s.clone());
        if let Some(port_name) = port_name {
            match port_name.as_str() {
                "a" => vec![&self.a, &self.b],
                "b" => vec![&self.a, &self.b],
                "y" => vec![&self.y],
                _ => vec![],
            }
        } else {
            vec![]
        }
    }
}