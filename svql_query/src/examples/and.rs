use crate::module::{lookup, QueryError, RtlModuleResultTrait, RtlModuleTrait};
use crate::ports::{InPort, OutPort};
use std::collections::HashMap;
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

impl RtlModuleTrait<And> for And {
    fn file_path(&self) -> PathBuf {
        "svql_query/verilog/and.v".into()
    }

    fn module_name(&self) -> &'static str {
        "and_gate"
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

impl RtlModuleResultTrait<AndResult> for AndResult {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Result<AndResult, QueryError> {
        Ok(AndResult {
            a: lookup(&port_map, "a")?,
            b: lookup(&port_map, "b")?,
            y: lookup(&port_map, "y")?,
        })
    }
}
