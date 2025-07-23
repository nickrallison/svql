use crate::examples::and::{And, AndResult};
use crate::module::{RtlModule, RtlQueryResultTrait, RtlQueryTrait};
use std::collections::HashSet;

use crate::connect;
use crate::ports::{Connection, InPort, OutPort};
use std::fmt::Debug;

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
        connect!(&mut connections, &self.and1.module.y, &self.and2.module.a);
        connect!(&mut connections, &self.and2.module.y, &self.and3.module.a);
        connections
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
