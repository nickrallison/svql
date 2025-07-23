use std::{net::ToSocketAddrs, path::PathBuf};
use std::collections::HashSet;

use crate::{
    connection::Connection,
    ports::{InPort, OutPort},
    query::Module,
};

mod query;
mod ports;
mod connection;
mod net;

#[derive(Debug)]
struct And {
    inst: String,
    a: InPort,
    b: InPort,
    y: OutPort,
}
impl And {
    fn new(inst: &str) -> Self {
        And {
            inst: inst.into(),
            a: InPort::new(format!("{inst}.a")),
            b: InPort::new(format!("{inst}.b")),
            y: OutPort::new(format!("{inst}.y")),
        }
    }

    fn from_module_path(
        mut self,
        module_path: &str
    ) -> Self {
        And {
            inst: self.inst.clone(),
            a: InPort::new(format!("{module_path}.{}.a", self.inst.clone())),
            b: InPort::new(format!("{module_path}.{}.b", self.inst.clone())),
            y: OutPort::new(format!("{module_path}.{}.y", self.inst.clone())),
        }
    }
}
impl Module for And {
    fn file_path(&self)   -> PathBuf { "svql_query/verilog/and.v".into() }
    fn module_name(&self) -> String  { "and_gate".into() }
    fn instance_name(&self) -> &str  { &self.inst }
}

// ── composite ───────────────────────────────────────
#[derive(Debug)]
struct SubCombinedAnd {
    inst: String,
    connections: HashSet<Connection>,
    and1: And,
    and2: And,
}
impl SubCombinedAnd {
    fn new(inst: &str, and1: And, and2: And) -> Self {
        
        SubCombinedAnd {
            inst: inst.into(),
            connections: HashSet::new(),
            and1: and1.from_module_path(
                inst,
            ),
            and2: and2.from_module_path(
                inst,
            ),
        }
    }
    fn connect(mut self) -> Self {
        connect!(self, &self.and2.a, &self.and1.y);
        self
    }

    fn from_module_path(mut self, module_path: &str) -> Self {
        // build "<module_path>.<self.inst>"
        let prefix = if module_path.is_empty() {
            self.inst.clone()
        } else {
            format!("{module_path}.{}", self.inst)
        };

        // rename the leaf modules
        self.and1 = self.and1.from_module_path(&prefix);
        self.and2 = self.and2.from_module_path(&prefix);

        // the old connections still point to the old names – drop them
        self.connections.clear();

        // and recreate them with the new names
        self = self.connect();
        self
    }
}
impl Module for SubCombinedAnd {
    // dummy leaf info – unused because this is a composite
    fn file_path(&self)   -> PathBuf { PathBuf::new() }
    fn module_name(&self) -> String  { "SubCombinedAnd".into() }
    fn instance_name(&self) -> &str  { "SubCombinedAnd" }

    fn submodules(&self) -> Vec<&dyn Module> {
        vec![&self.and1, &self.and2]
    }
    fn connections(&self) -> &HashSet<Connection> { &self.connections }
}

// ── composite ───────────────────────────────────────
#[derive(Debug)]
struct CombinedAnd {
    inst: String,
    connections: HashSet<Connection>,
    and: And,
    sub_combined_and: SubCombinedAnd,
}
impl CombinedAnd {
    fn new(inst: &str, and: And, sub_combined_and: SubCombinedAnd) -> Self {
        CombinedAnd {
            inst: inst.into(),
            connections: HashSet::new(),
            and: and.from_module_path(
                inst,
            ),
            sub_combined_and: sub_combined_and.from_module_path(
                inst,
            ),
        }
    }
    fn connect(mut self) -> Self {
        self.sub_combined_and = self.sub_combined_and
            .connect();
        connect!(self, &self.sub_combined_and.and2.y, &self.and.a);
        self
    }
}
impl Module for CombinedAnd {
    // dummy leaf info – unused because this is a composite
    fn file_path(&self)   -> PathBuf { PathBuf::new() }
    fn module_name(&self) -> String  { "CombinedAnd".into() }
    fn instance_name(&self) -> &str  { "CombinedAnd" }

    fn submodules(&self) -> Vec<&dyn Module> {
        vec![&self.and, &self.sub_combined_and]
    }
    fn connections(&self) -> &HashSet<Connection> { &self.connections }
}

fn main() {
    let and1 = And::new("and1");

    // object-safe call
    let res = and1.query_net("127.0.0.1:9999").unwrap();
    for m in &res { println!("{}", m); }
    println!("---");

    let and2 = And::new("and2");
    let and3 = And::new("and");

    let mut sub_combined = SubCombinedAnd::new("SubCombinedAnd", and1, and2);
    let mut combined = CombinedAnd::new("CombinedAnd", and3, sub_combined).connect();

    println!("CombinedAnd: {:#?}", combined);

    let res2 = combined.query_net("127.0.0.1:9999").unwrap();

    println!("---");
    println!("CombinedAnd query result:");
    for m in &res2 { println!("{}", m); }
}