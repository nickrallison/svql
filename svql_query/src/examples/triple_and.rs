use crate::examples::and::{And, AndResult};
use crate::module::{RtlModule, RtlQueryResultTrait, RtlQueryTrait};
use std::collections::HashSet;

use crate::ports::{Connection, InPort, OutPort};
use std::fmt::Debug;
use svql_common::mat::{IdString, SanitizedQueryMatch};

#[derive(Debug, Clone)]
pub struct TripleAnd {
    pub and1: RtlModule<And>,
    pub and2: RtlModule<And>,
    pub and3: RtlModule<And>,
}

impl TripleAnd {
    pub fn new() -> Self {
        TripleAnd {
            and1: RtlModule::new("and1".into(), And::new()),
            and2: RtlModule::new("and2".into(), And::new()),
            and3: RtlModule::new("and3".into(), And::new()),
        }
    }
}

impl RtlQueryTrait for TripleAnd {
    type Result = TripleAndResult;

    fn connect(&self) -> HashSet<Connection<InPort, OutPort>> {
        let mut connections = HashSet::new();
        // Try using hierarchical names for the ports
        connections.insert(Connection::new(
            InPort::new("and2.a"),
            OutPort::new("and1.y"),
        ));
        connections.insert(Connection::new(
            InPort::new("and3.a"),
            OutPort::new("and2.y"),
        ));

        // Print connections for debugging
        log::trace!("TripleAnd connections:");
        for conn in &connections {
            log::trace!("  {} -> {}", conn.out_port.0, conn.in_port.0);
        }

        connections
    }

    fn sub_modules(&self) -> Vec<&dyn Queryable> {
        vec![&self.and1, &self.and2, &self.and3]
    }

    fn sub_queries(&self) -> Vec<&dyn Queryable> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct TripleAndResult {
    pub and1: AndResult,
    pub and2: AndResult,
    pub and3: AndResult,
}

impl TripleAndResult {
    pub fn new(and1: AndResult, and2: AndResult, and3: AndResult) -> Self {
        TripleAndResult { and1, and2, and3 }
    }
}

impl RtlQueryResultTrait for TripleAndResult {}
