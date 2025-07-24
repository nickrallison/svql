use crate::examples::and::{And, AndResult};
use crate::module::{Queryable, RtlModule, RtlQueryResultTrait, RtlQueryTrait};
use std::collections::HashSet;

use crate::connect;
use crate::ports::{Connection, InPort, OutPort};
use std::fmt::Debug;
use svql_common::mat::{IdString, SanitizedQueryMatch};

#[derive(Debug, Clone)]
pub struct DoubleAnd {
    pub and1: RtlModule<And>,
    pub and2: RtlModule<And>,
}

impl DoubleAnd {
    pub fn new() -> Self {
        DoubleAnd {
            and1: RtlModule::new("and1".into(), And::new()),
            and2: RtlModule::new("and2".into(), And::new()),
        }
    }
}

impl RtlQueryTrait for DoubleAnd {
    type Result = DoubleAndResult;

    fn connect(&self) -> HashSet<Connection<InPort, OutPort>> {
        let mut connections = HashSet::new();
        // Try using hierarchical names for the ports
        connect!(connections, &self.and1.module.y, &self.and2.module.a);

        // Print connections for debugging
        log::trace!("DoubleAnd connections:");
        for conn in &connections {
            log::trace!("  {} -> {}", conn.out_port.0, conn.in_port.0);
        }

        connections
    }

    fn sub_modules(&self) -> Vec<&dyn Queryable> {
        vec![&self.and1, &self.and2]
    }

    fn sub_queries(&self) -> Vec<&dyn Queryable> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct DoubleAndResult {
    pub and1: RtlModule<AndResult>,
    pub and2: RtlModule<AndResult>,
}

impl DoubleAndResult {
    pub fn new(and1: RtlModule<AndResult>, and2: RtlModule<AndResult>) -> Self {
        DoubleAndResult { and1, and2 }
    }
}

impl RtlQueryResultTrait for DoubleAndResult {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::mock::MockDriver;
    use crate::driver::Driver;
    use crate::module::RtlQuery;

    #[test]
    fn test_double_and() {
        let double_and: RtlQuery<DoubleAnd> =
            RtlQuery::new("double_and".to_string(), DoubleAnd::new());

        let driver = Driver::Mock(MockDriver);

        let matches = double_and.query(&driver).unwrap();
    }
}
