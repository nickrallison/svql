use crate::instance::Instance;
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
// Base Search Types
// ########################

pub struct Search;
pub struct Match;

// ########################
// Containers
// ########################

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wire<T> {
    pub path: Instance
    pub val: T,
}

impl <T> Wire<T> where T: Default {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path, name)
    }
    pub fn new(parent_path: Instance) -> Self {
        let path = path.child(name);
        Self { path, val: T::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct And<T> {
    pub path: Instance,
    pub a: Wire<T>,
    pub b: Wire<T>,
    pub y: Wire<T>,
}

impl<S, M> And<S> {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path, name)
    }
    pub fn new(parent_path: Instance) -> Self {
        let a = Wire::new(parent_path.child("a".to_string()));
        let b = Wire::new(parent_path.child("b".to_string()));
        let y = Wire::new(parent_path.child("y".to_string()));
        Self { path: parent_path, a, b, y }
    }
    pub fn query(&self) -> Vec<And<M>> {
        todo!();
    }
}

impl And<Search> {
    pub fn query(&self) -> Vec<And<Match>> {
        // This is a placeholder for the actual query logic
        // In a real implementation, this would interact with a database or other data source
        Ok(vec![And::new(self.path.clone())])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubleAnd<T> {
    pub path: Instance,
    pub and1: And<T>,
    pub and2: And<T>,
}

impl<T> DoubleAnd<T> {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path, name)
    }
    pub fn new(parent_path: Instance) -> Self {
        let and1 = And::new(parent_path.child("and1".to_string()));
        let and2 = And::new(parent_path.child("and2".to_string()));
        Self { path: parent_path, and1, and2 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripleAnd<T> {
    pub path: Instance,
    pub double_and: DoubleAnd<T>,
    pub and: And<T>,
}

impl<T> TripleAnd<T> {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path, name)
    }
    pub fn new(parent_path: Instance) -> Self {
        let double_and = DoubleAnd::new(parent_path.child("double_and".to_string()));
        let and = And::new(parent_path.child("and".to_string()));
        Self { path: parent_path, double_and, and }
    }
}

fn main() {
    let and_search: And<Search> = And::root("and");
    let double_and_search: DoubleAnd<Search> = DoubleAnd::root("double_and");
    let triple_and_search: TripleAnd<Search> = TripleAnd::root("triple_and");

    let and_matches: Vec<And<Match>> = and_search.query().unwrap();
    let double_and_matches: Vec<DoubleAnd<Match>> = double_and_search.query().unwrap();
    let triple_and_matches: Vec<TripleAnd<Match>> = triple_and_search.query().unwrap();
    //... 
}

impl And<T> {
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
}

impl And<Search> {
    fn initialize(path: Instance) -> Self {
        Self {
            a: InPort::new("a", path.clone()),
            b: InPort::new("b", path.clone()),
            y: OutPort::new("y", path.clone()),
        }
    }

    fn root() -> Self {
        let path = Instance::root(Self::module_name());
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