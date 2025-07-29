use crate::examples::and::{And, AndResult};
use crate::examples::double_and::{DoubleAnd, DoubleAndResult};
use crate::module::RtlModule;
use crate::query::RtlQuery;
use std::collections::{HashSet, VecDeque};

use crate::connect;
use crate::driver::{Driver, DriverError};
use crate::module::result::RtlModuleResult;
use crate::module::traits::RtlModuleResultTrait;
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use crate::query::traits::{RtlQueryResultTrait, RtlQueryTrait};
use itertools::iproduct;
use std::fmt::Debug;
use std::sync::Arc;
use svql_common::matches::IdString;

#[derive(Debug, Clone)]
pub struct TripleAnd {
    pub and: RtlModule<And>,
    pub double_and: RtlQuery<DoubleAnd>,
}

impl TripleAnd {
    pub fn new() -> Self {
        TripleAnd {
            and: RtlModule::new(And::new(), "and".into()),
            double_and: RtlQuery::new(DoubleAnd::new(), "double_and".into()),
        }
    }
}

impl RtlQueryTrait for TripleAnd {
    type Result = TripleAndResult;

    fn connect(&self) -> HashSet<Connection<InPort, OutPort>> {
        let mut connections = HashSet::new();
        // Try using hierarchical names for the ports
        connect!(connections, &self.double_and.query.and2.module.y, &self.and.module.a);
        connections
    }

    fn init_full_path(&mut self, full_path: VecDeque<Arc<String>>) {
        // Initialize full path for both AND gates
        self.double_and.init_full_path(full_path.clone());
        self.and.init_full_path(full_path.clone());
    }

    fn query(
        &self,
        driver: &Driver,
        inst: Arc<String>,
        full_path: VecDeque<Arc<String>>,
    ) -> Result<Vec<RtlQueryResult<Self::Result>>, DriverError> {
        // Get the query iterators for both AND gates

        let and_results = self.and.query(driver)?;
        let double_and_results = self.double_and.query(driver)?;

        // Create a cartesian product of the results
        let cartesian_product = iproduct!(and_results, double_and_results);

        // Map the cartesian product to TripleAndResult instances
        let matches = cartesian_product.map(|(and_result, double_and_result)| {
            let triple_and_result = TripleAndResult::new(double_and_result, and_result);
            RtlQueryResult::new(triple_and_result, inst.clone(), full_path.clone())
        });

        let filtered_matches: Vec<RtlQueryResult<Self::Result>> = matches
            .filter(|match_result| {
                // Check if the match is valid based on the connections
                match_result.query.validate_connections(&self.connect())
            })
            .collect();

        Ok(filtered_matches)
    }
}

#[derive(Debug, Clone)]
pub struct TripleAndResult {
    pub double_and: RtlQueryResult<DoubleAndResult>,
    pub and: RtlModuleResult<AndResult>,
}

impl TripleAndResult {
    pub fn new(double_and: RtlQueryResult<DoubleAndResult>, and: RtlModuleResult<AndResult>) -> Self {
        TripleAndResult { double_and, and }
    }
}

impl RtlQueryResultTrait for TripleAndResult {
    fn validate_connection(&self, connection: &Connection<InPort, OutPort>) -> bool {
        let in_port_id = self.find_port(connection.in_port.full_path.clone());
        let out_port_id = self.find_port(connection.out_port.full_path.clone());

        if let (Some(in_port), Some(out_port)) = (in_port_id, out_port_id) {
            return in_port == out_port;
        }
        false
    }

    fn find_port(&self, mut port_name: VecDeque<Arc<String>>) -> Option<&IdString> {
        let _top_str_option = port_name.pop_front();

        let peeked_str_option = port_name.iter().next();
        if let Some(peeked_str) = peeked_str_option {
            // for module in modules, check if the port matches
            if *peeked_str == self.double_and.inst {
                return self.double_and.query.find_port(port_name);
            } else if *peeked_str == self.and.inst {
                return self.and.module.find_port(port_name);
            }
        }
        None
    }
}

#[cfg(test)]
mod triple_and_tests {
    use super::*;
    use crate::driver::mock::MockDriver;
    use crate::driver::Driver;
    use crate::ports;
    use crate::query::RtlQuery;

    #[test]
    fn test_port_path() {
        let triple_and: RtlQuery<TripleAnd> =
            RtlQuery::new(TripleAnd::new(), "triple_and".to_string());

        let port = triple_and.query.double_and.query.and1.module.a.clone();

        assert_eq!(port.inst_path(), "triple_and.double_and.and1.a".to_string());
    }

    #[test]
    fn test_matches_len() {
        let triple_and: RtlQuery<TripleAnd> =
            RtlQuery::new(TripleAnd::new(), "triple_and".to_string());

        let driver = Driver::Mock(MockDriver);

        let matches = triple_and.query(&driver).unwrap();
        println!("TripleAnd matches len: {:?}", matches.len());
        assert!(matches.len() == 1, "Expected 1 matches for TripleAnd query");
    }
}
