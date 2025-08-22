use log::trace;
// svql_query/src/composite.rs
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
    type Hit<'p, 'd>;

    fn context(driver: &Driver) -> Result<Context, Box<dyn std::error::Error>>;

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx, 'ctx>>;
}

pub trait MatchedComposite<'p, 'd>: Composite<Match<'p, 'd>> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool + '_>> {
        vec![]
    }

    fn validate_connection(&self, connection: Connection<Match<'p, 'd>>) -> bool {
        let in_port_id = self.find_port(&connection.from.path);
        let out_port_id = self.find_port(&connection.to.path);

        trace!(
            "Validating connection: from={:?} to={:?}",
            connection.from.path, connection.to.path
        );
        trace!("Found from port: {:?}", in_port_id);
        trace!("Found to port: {:?}", out_port_id);

        if let (Some(in_port), Some(out_port)) = (in_port_id, out_port_id) {
            let from_match = &in_port.val;
            let to_match = &out_port.val;

            trace!("From match: {:?}", from_match);
            trace!("To match: {:?}", to_match);

            if let (Some(from_val), Some(to_val)) = (from_match, to_match) {
                let from_cell = from_val.design_cell_ref;
                let to_cell = to_val.design_cell_ref;

                trace!("From cell: {:?}", from_cell);
                trace!("To cell: {:?}", to_cell);

                let result = from_cell.is_some() && to_cell.is_some() && from_cell == to_cell;
                trace!("Connection validation result: {}", result);
                return result;
            }
        }
        trace!("Connection validation failed - missing ports or matches");
        false
    }

    fn validate_connections(&self, connections: Vec<Vec<Connection<Match<'p, 'd>>>>) -> bool {
        trace!("Validating connections for composite: {:?}", self.path());
        for (i, connection_set) in connections.iter().enumerate() {
            trace!("Checking connection set {}: {:?}", i, connection_set);
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
