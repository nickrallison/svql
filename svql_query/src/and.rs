use std::collections::HashSet;
// and.rs
use crate::module::{Module, ModuleRaw};
use svql_common::mat::{
    SanitizedQueryMatch, IdString, SanitizedCellData,
};
use crate::net::SvqlQueryError;
use std::path::PathBuf;
use crate::ports::{Connection, InPort, OutPort};

#[derive(Debug)]
pub struct And {
    pub a: InPort,
    pub b: InPort,
    pub y: OutPort,

    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
}

impl And {
    pub fn new(
        inst: String,
    ) -> Self {
        And {
            a: InPort::new("a"),
            b: InPort::new("b"),
            y: OutPort::new("y"),
            inst,
            connections: HashSet::new(),
        }
    }

    pub fn add_connection(
        &mut self,
        conn: Connection<InPort, OutPort>,
    ) {
        self.connections.insert(conn);
    }
}

#[derive(Debug)]
pub struct AndResult {
    pub a     : IdString,
    pub b     : IdString,
    pub y     : IdString,

    pub inst  : String,
    pub cells : Vec<SanitizedCellData>,
}

// ───────── ModuleRaw
impl ModuleRaw for And {
    fn file_path(&self)   -> PathBuf     { "svql_query/verilog/and.v".into() }
    fn module_name(&self) -> &'static str{ "and_gate" }

    fn query_raw(
        &self,
        addr: &str,
    ) -> Result<Vec<SanitizedQueryMatch>, SvqlQueryError> {
        use crate::query::run_svql_query_leaf;
        run_svql_query_leaf(addr, self.file_path(), self.module_name().to_string())
    }
}

impl Module for And {
    type Result = AndResult;

    fn from_match(
        &self,
        m: SanitizedQueryMatch,
    ) -> Result<Self::Result, SvqlQueryError> {
        Ok(AndResult {
            inst  : self.inst.clone(),
            a     : lookup(&m, "a")?,
            b     : lookup(&m, "b")?,
            y     : lookup(&m, "y")?,
            cells : m.cell_map.into_values().collect(),
        })
    }
}

// helper
fn lookup(
    m   : &SanitizedQueryMatch,
    pin : &str,
) -> Result<IdString, SvqlQueryError> {
    m.port_map
        .get(&IdString::Named(pin.into()))
        .cloned()
        .ok_or_else(|| SvqlQueryError::ResponseError(
            format!("missing port `{pin}`")
        ))
}