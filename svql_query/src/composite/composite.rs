use svql_common::Config;
use svql_driver::{DriverKey, context::Context, driver::Driver};

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
        tracing::event!(
            tracing::Level::TRACE,
            "Validating connection: from={:?} to={:?}",
            connection.from.path,
            connection.to.path
        );

        // Find the actual wire ports in the composite structure
        let from_wire = self.find_port(&connection.from.path);
        let to_wire = self.find_port(&connection.to.path);

        tracing::event!(
            tracing::Level::TRACE,
            "Found from port: {:?}",
            from_wire.is_some()
        );
        tracing::event!(
            tracing::Level::TRACE,
            "Found to port: {:?}",
            to_wire.is_some()
        );

        match (from_wire, to_wire) {
            (Some(from), Some(to)) => {
                let from_match = &from.val;
                let to_match = &to.val;

                tracing::event!(
                    tracing::Level::TRACE,
                    "From match present: {:?}",
                    from_match.is_some()
                );
                tracing::event!(
                    tracing::Level::TRACE,
                    "To match present: {:?}",
                    to_match.is_some()
                );

                match (from_match, to_match) {
                    (Some(from_val), Some(to_val)) => {
                        let from_cell = from_val.design_cell_ref.as_ref();
                        let to_cell = to_val.design_cell_ref.as_ref();

                        tracing::event!(
                            tracing::Level::TRACE,
                            "From cell present: {:?}",
                            from_cell.is_some()
                        );
                        tracing::event!(
                            tracing::Level::TRACE,
                            "To cell present: {:?}",
                            to_cell.is_some()
                        );

                        if let (Some(from_c), Some(to_c)) = (from_cell, to_cell) {
                            let result = from_c == to_c;
                            tracing::event!(tracing::Level::TRACE, "Cells equal: {}", result);
                            result
                        } else {
                            tracing::event!(
                                tracing::Level::TRACE,
                                "Connection validation failed - missing cell references"
                            );
                            false
                        }
                    }
                    _ => {
                        tracing::event!(
                            tracing::Level::TRACE,
                            "Connection validation failed - missing match values"
                        );
                        false
                    }
                }
            }
            _ => {
                tracing::event!(
                    tracing::Level::TRACE,
                    "Connection validation failed - could not find ports"
                );
                false
            }
        }
    }

    /// Validate all connection sets - at least one connection in each set must be valid
    fn validate_connections(&self, connections: Vec<Vec<Connection<Match<'ctx>>>>) -> bool {
        tracing::event!(
            tracing::Level::TRACE,
            "Validating {} connection sets for composite",
            connections.len()
        );
        for (i, connection_set) in connections.iter().enumerate() {
            tracing::event!(
                tracing::Level::TRACE,
                "Checking connection set {}: {} connections",
                i,
                connection_set.len()
            );
            let mut valid = false;
            for conn in connection_set {
                if self.validate_connection(conn.clone()) {
                    tracing::event!(tracing::Level::TRACE, "Connection set {} is valid", i);
                    valid = true;
                    break;
                }
            }
            if !valid {
                tracing::event!(
                    tracing::Level::TRACE,
                    "Connection set {} failed validation",
                    i
                );
                return false;
            }
        }
        tracing::event!(
            tracing::Level::TRACE,
            "All connection sets passed validation"
        );
        true
    }
}
