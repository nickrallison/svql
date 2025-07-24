use crate::examples::and::{And, AndResult};
use crate::module::RtlModule;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::connect;
use crate::driver::{Driver, DriverError};
use crate::module::result::RtlModuleResult;
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use crate::query::traits::{RtlQueryResultTrait, RtlQueryTrait};
use itertools::iproduct;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DoubleAnd {
    pub and1: RtlModule<And>,
    pub and2: RtlModule<And>,
}

impl DoubleAnd {
    pub fn new() -> Self {
        DoubleAnd {
            and1: RtlModule::new(And::new(), "and1".into()),
            and2: RtlModule::new(And::new(), "and2".into()),
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
            let out_port_path: String = conn
                .out_port
                .full_path
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(".");
            let in_port_path: String = conn
                .in_port
                .full_path
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(".");
            log::trace!("  {} -> {}", out_port_path, in_port_path);
        }

        connections
    }

    fn query(
        &self,
        driver: &Driver,
        inst: Arc<String>,
        full_path: Vec<Arc<String>>,
    ) -> Result<Vec<RtlQueryResult<Self::Result>>, DriverError> {
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

            RtlQueryResult::new(double_and_result, inst.clone(), full_path.clone())
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
    // fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self {
    //     todo!()
    // }
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
            RtlQuery::new(DoubleAnd::new(), "double_and".to_string());

        let inst = double_and.inst.clone();
        let inst_path = double_and.full_path.clone();

        let driver = Driver::Mock(MockDriver);

        let matches = double_and.query.query(&driver, inst, inst_path).unwrap();
        log::debug!("DoubleAnd matches: {:?}", matches);
        // assert!(matches.len() == 2, "Expected 2 matches for DoubleAnd query");
    }
}
