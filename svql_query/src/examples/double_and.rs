use crate::examples::and::{And, AndResult};
use crate::module::RtlModule;
use std::collections::{HashMap, HashSet};

use crate::connect;
use crate::driver::{Driver, DriverError};
use crate::module::result::RtlModuleResult;
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use crate::query::traits::{RtlQueryResultTrait, RtlQueryTrait};
use itertools::iproduct;
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

    fn query(&self, driver: &Driver) -> Result<Vec<RtlQueryResult<Self::Result>>, DriverError> {
        // // Get the query iterators for both AND gates
        let and1_results = self.and1.query(driver)?;
        let and2_results = self.and2.query(driver)?;
        //
        // // Create a cartesian product of the results
        let cartesian_product = iproduct!(and1_results, and2_results);
        //
        // // Map the cartesian product to DoubleAndResult instances
        let matches = cartesian_product.map(|(and1_result, and2_result)| {
            let double_and_result = DoubleAndResult::new(and1_result, and2_result);
            RtlQueryResult::new(double_and_result)
        });
        let filtered_matches: Vec<RtlQueryResult<Self::Result>> = matches
            .filter(|match_result| {
                // Check if the match is valid based on the connections
                match_result.query.valid_connections(&self.connect())
            })
            .collect();

        Ok(filtered_matches)
    }
}

#[derive(Debug, Clone)]
pub struct DoubleAndResult {
    pub and1: RtlModuleResult<AndResult>,
    pub and2: RtlModuleResult<AndResult>,
}

impl DoubleAndResult {
    pub fn new(and1: RtlModuleResult<AndResult>, and2: RtlModuleResult<AndResult>) -> Self {
        DoubleAndResult { and1, and2 }
    }
}

impl RtlQueryResultTrait for DoubleAndResult {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::mock::MockDriver;
    use crate::driver::Driver;
    use crate::query::RtlQuery;

    #[test]
    fn test_double_and() {
        let double_and: RtlQuery<DoubleAnd> =
            RtlQuery::new("double_and".to_string(), DoubleAnd::new());

        let driver = Driver::Mock(MockDriver);

        let matches = double_and.query.query(&driver).unwrap();
    }
}
