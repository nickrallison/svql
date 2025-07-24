use crate::driver::{Driver, DriverError};
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

pub trait RtlQueryTrait {
    /// Type produced for every successful match of this query.
    type Result: Debug + RtlQueryResultTrait;

    /// The set of extra connections the query wants to impose.
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;
    fn init_full_path(&mut self, full_path: Vec<Arc<String>>);
    fn query(
        &self,
        driver: &Driver,
        inst: Arc<String>,
        full_path: Vec<Arc<String>>,
    ) -> Result<Vec<RtlQueryResult<Self::Result>>, DriverError>;
}

pub trait RtlQueryResultTrait {
    // fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self;
    fn valid_connections(&self, connections: &HashSet<Connection<InPort, OutPort>>) -> bool {
        // Default implementation can be overridden if needed
        // todo!();
        true
    }
}
