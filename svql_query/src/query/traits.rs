use svql_common::id_string::IdString;

use crate::driver::{Driver, DriverError};
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;

pub trait RtlQueryTrait {
    /// Type produced for every successful match of this query.
    type Result: Debug + RtlQueryResultTrait;

    /// The set of extra connections the query wants to impose.
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;
    fn init_full_path(&mut self, full_path: VecDeque<Arc<String>>, height: usize);
    fn query(
        &self,
        driver: &Driver,
        inst: Arc<String>,
        full_path: VecDeque<Arc<String>>,
        height: usize
    ) -> Result<Vec<RtlQueryResult<Self::Result>>, DriverError>;
}

pub trait RtlQueryResultTrait {
    fn validate_connections(&self, connections: &HashSet<Connection<InPort, OutPort>>, height: usize) -> bool {
        for conn in connections {
            if !self.validate_connection(conn, height) {
                return false;
            }
        }
        true
    }
    fn validate_connection(&self, connection: &Connection<InPort, OutPort>, height: usize) -> bool;
    fn find_port(&self,port_name: VecDeque<Arc<String>>, height: usize) -> Option<&IdString>;
}
