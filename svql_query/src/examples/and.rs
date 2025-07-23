// and.rs
use crate::module::{lookup, QueryError, RtlModule};
use crate::ports::{Connection, InPort, OutPort};
use std::collections::HashSet;
use std::path::PathBuf;
use svql_common::mat::{IdString, SanitizedCellData, SanitizedQueryMatch};

#[derive(Debug)]
pub struct And {
    pub a: InPort,
    pub b: InPort,
    pub y: OutPort,

    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
}

impl And {
    pub fn new(inst: String) -> Self {
        And {
            a: InPort::new("a"),
            b: InPort::new("b"),
            y: OutPort::new("y"),
            inst,
            connections: HashSet::new(),
        }
    }

    pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
        self.connections.insert(conn);
    }
}

#[derive(Debug)]
pub struct AndResult {
    pub a: IdString,
    pub b: IdString,
    pub y: IdString,

    pub inst: String,
    pub cells: Vec<SanitizedCellData>,
}

// ───────── ModuleRaw
impl RtlModule for And {
    type Result = AndResult;

    fn instance(&self, parent: Option<&str>) -> String {
        if parent.is_none() {
            return self.inst.clone();
        }
        format!("{}.{}", parent.unwrap(), self.inst)
    }

    fn file_path(&self) -> PathBuf {
        "svql_query/verilog/and.v".into()
    }
    fn module_name(&self) -> &'static str {
        "and_gate"
    }

    fn from_match(m: SanitizedQueryMatch, inst: String) -> Result<Self::Result, QueryError> {
        Ok(AndResult {
            inst,
            a: lookup(&m, "a")?,
            b: lookup(&m, "b")?,
            y: lookup(&m, "y")?,
            cells: m.cell_map.into_values().collect(),
        })
    }
}
