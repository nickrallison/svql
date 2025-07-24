use crate::driver::{Driver, DriverError};
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use svql_common::mat::IdString;

pub trait RtlQueryTrait {
    /// Type produced for every successful match of this query.
    type Result: Debug + RtlQueryResultTrait;

    /// The set of extra connections the query wants to impose.
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;
    fn query(
        &self,
        driver: &Driver,
    ) -> Result<Box<dyn Iterator<Item = RtlQueryResult<Self::Result>> + '_>, DriverError>;
}

pub trait RtlQueryResultTrait {
    fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self;
}
