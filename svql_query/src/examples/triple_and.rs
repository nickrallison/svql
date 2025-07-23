// and.rs
use crate::examples::and::{And, AndResult};
use crate::ports::{Connection, InPort, OutPort};
use std::collections::HashSet;
use svql_common::mat::SanitizedCellData;

#[derive(Debug)]
pub struct TripleAnd {
    pub and1: And,
    pub and2: And,
    pub and3: And,

    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
}

impl TripleAnd {
    pub fn new(inst: String) -> Self {
        TripleAnd {
            and1: And::new("and1".into()),
            and2: And::new("and2".into()),
            and3: And::new("and3".into()),
            inst,
            connections: HashSet::new(),
        }
    }

    pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
        self.connections.insert(conn);
    }
}

#[derive(Debug)]
pub struct TripleAndResult {
    pub and1: AndResult,
    pub and2: AndResult,
    pub and3: AndResult,

    pub inst: String,
    pub cells: Vec<SanitizedCellData>,
}
