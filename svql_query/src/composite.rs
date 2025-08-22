// svql_query/src/composite.rs
use log::trace;
use svql_driver::{DriverKey, context::Context, driver::Driver};
use svql_subgraph::Config;

use crate::{Connection, Match, Search, State, WithPath, instance::Instance};

pub trait Composite<S>: WithPath<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>>;
}

pub trait SearchableComposite: Composite<Search> {
    type Hit<'ctx>;

    fn context(driver: &Driver) -> Result<Context, Box<dyn std::error::Error>>;

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>>;
}

pub trait MatchedComposite<'ctx>: Composite<Match<'ctx>> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool + '_>> {
        vec![]
    }

    /// Validate that a connection represents a valid design connectivity
    /// where the source and destination refer to the same design cell/bit
    fn validate_connection(&self, connection: Connection<Match<'ctx>>) -> bool {
        trace!(
            "Validating connection: from={:?} to={:?}",
            connection.from.path, connection.to.path
        );

        // Find the actual wire ports in the composite structure
        let from_wire = self.find_port(&connection.from.path);
        let to_wire = self.find_port(&connection.to.path);

        trace!("Found from port: {:?}", from_wire.is_some());
        trace!("Found to port: {:?}", to_wire.is_some());

        match (from_wire, to_wire) {
            (Some(from), Some(to)) => {
                let from_match = &from.val;
                let to_match = &to.val;

                trace!("From match present: {:?}", from_match.is_some());
                trace!("To match present: {:?}", to_match.is_some());

                match (from_match, to_match) {
                    (Some(from_val), Some(to_val)) => {
                        let from_cell = from_val.design_cell_ref;
                        let to_cell = to_val.design_cell_ref;

                        trace!("From cell present: {:?}", from_cell.is_some());
                        trace!("To cell present: {:?}", to_cell.is_some());

                        if let (Some(from_c), Some(to_c)) = (from_cell, to_cell) {
                            let result = from_c == to_c;
                            trace!("Cells equal: {}", result);
                            result
                        } else {
                            trace!("Connection validation failed - missing cell references");
                            false
                        }
                    }
                    _ => {
                        trace!("Connection validation failed - missing match values");
                        false
                    }
                }
            }
            _ => {
                trace!("Connection validation failed - could not find ports");
                false
            }
        }
    }

    /// Validate all connection sets - at least one connection in each set must be valid
    fn validate_connections(&self, connections: Vec<Vec<Connection<Match<'ctx>>>>) -> bool {
        trace!(
            "Validating {} connection sets for composite",
            connections.len()
        );
        for (i, connection_set) in connections.iter().enumerate() {
            trace!(
                "Checking connection set {}: {} connections",
                i,
                connection_set.len()
            );
            let mut valid = false;
            for conn in connection_set {
                if self.validate_connection(conn.clone()) {
                    trace!("Connection set {} is valid", i);
                    valid = true;
                    break;
                }
            }
            if !valid {
                trace!("Connection set {} failed validation", i);
                return false;
            }
        }
        trace!("All connection sets passed validation");
        true
    }
}
