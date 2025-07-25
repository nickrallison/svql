use crate::driver::{Driver, DriverError};
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use svql_common::matches::IdString;

pub trait RtlQueryTrait {
    /// Type produced for every successful match of this query.
    type Result: Debug + RtlQueryResultTrait;

    /// The set of extra connections the query wants to impose.
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;
    fn init_full_path(&mut self, full_path: VecDeque<Arc<String>>);
    fn query(
        &self,
        driver: &Driver,
        inst: Arc<String>,
        full_path: VecDeque<Arc<String>>,
    ) -> Result<Vec<RtlQueryResult<Self::Result>>, DriverError>;
}

pub trait RtlQueryResultTrait {
    fn validate_connections(&self, connections: &HashSet<Connection<InPort, OutPort>>) -> bool {
        for conn in connections {
            if !self.validate_connection(conn) {
                return false;
            }
        }
        true
    }
    fn validate_connection(&self, connections: &Connection<InPort, OutPort>) -> bool;
    fn find_port(&self, port_name: VecDeque<Arc<String>>) -> Option<&IdString>;
}
