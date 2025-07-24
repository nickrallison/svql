use crate::driver::{Driver, DriverError};
use crate::ports::{Connection, InPort, OutPort};
use crate::query::result::RtlQueryResult;
use std::collections::HashSet;
use std::fmt::Debug;

pub trait RtlQueryTrait<'a> {
    /// Type produced for every successful match of this query.
    type Result: Debug + RtlQueryResultTrait;

    /// The set of extra connections the query wants to impose.
    fn connect(&self) -> HashSet<Connection<InPort, OutPort>>;
    fn query(&self, driver: &Driver) -> Result<Vec<RtlQueryResult<Self::Result>>, DriverError>;
    fn set_parent(&mut self, parent: Option<&'a dyn RtlBoxedQueryTrait>);
}

pub trait RtlBoxedQueryTrait: Debug {
    fn instance(&self) -> &str;
    fn parent(&self) -> Option<&dyn RtlBoxedQueryTrait>;
}

pub trait RtlQueryResultTrait {
    // fn from_portmap(port_map: HashMap<IdString, IdString>) -> Self;
    fn valid_connections(&self, connections: &HashSet<Connection<InPort, OutPort>>) -> bool {
        // Default implementation can be overridden if needed
        todo!();
        true
    }
}
