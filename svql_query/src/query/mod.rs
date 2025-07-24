use crate::driver::{Driver, DriverError};
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use crate::query::traits::RtlQueryTrait;
use std::collections::HashSet;
use svql_common::config::ffi::SvqlRuntimeConfig;

pub mod result;
pub mod traits;

#[derive(Debug, Clone)]
pub struct RtlQuery<QueryType> {
    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
    pub query: QueryType,
}

impl<QueryType> RtlQuery<QueryType>
where
    QueryType: RtlQueryTrait,
{
    pub fn new(inst: String, query: QueryType) -> Self {
        RtlQuery {
            inst,
            connections: QueryType::connect(&query),
            query,
        }
    }

    fn instance(&self, parent: Option<&str>) -> String {
        crate::module::instance(&self.inst, parent)
    }

    pub fn add_connection(&mut self, conn: Connection<InPort, OutPort>) {
        self.connections.insert(conn);
    }

    pub(crate) fn config(&self) -> SvqlRuntimeConfig {
        let mut cfg = SvqlRuntimeConfig::default();
        // For queries, we'll need to define how to get the file path and module name
        // This might need to be added to RtlQueryTrait or handled differently
        cfg.verbose = true;
        cfg
    }

    pub fn query(
        &self,
        driver: &Driver,
    ) -> Result<Vec<RtlQueryResult<QueryType::Result>>, DriverError> {
        self.query.query(driver)
    }
}
