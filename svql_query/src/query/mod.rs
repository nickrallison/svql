use crate::driver::{Driver, DriverError};
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use crate::query::traits::{RtlBoxedQueryTrait, RtlQueryTrait};
use std::collections::HashSet;
use std::fmt::Debug;
use svql_common::config::ffi::SvqlRuntimeConfig;

pub mod result;
pub mod traits;

#[derive(Debug, Clone)]
pub struct RtlQuery<'a, QueryType> {
    pub inst: String,
    pub connections: HashSet<Connection<InPort, OutPort>>,
    pub query: QueryType,
    // #####
    pub parent: Option<&'a dyn RtlBoxedQueryTrait>,
}

impl<'a, QueryType> RtlQuery<'a, QueryType>
where
    QueryType: RtlQueryTrait<'a> + Debug,
{
    pub fn new(inst: String, query: QueryType, parent: Option<&'a dyn RtlBoxedQueryTrait>) -> Self {
        let mut query = {
            let mut query = RtlQuery {
                inst,
                connections: QueryType::connect(&query),
                query,
                parent,
            };
            query.query.set_parent(Some(&query));
            query
        };
        query
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

    fn set_parent(&mut self, parent: Option<&'a dyn RtlBoxedQueryTrait>) {
        self.parent = parent;
    }
}

impl<'a, QueryType> RtlBoxedQueryTrait for RtlQuery<'a, QueryType>
where
    QueryType: RtlQueryTrait<'a> + Debug,
{
    fn instance(&self) -> &str {
        self.inst.as_str()
    }
    fn parent(&self) -> Option<&'a dyn RtlBoxedQueryTrait> {
        self.parent
    }
}
