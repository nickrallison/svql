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

    fn context(&self, driver: &Driver) -> Context;

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

        if let (Some(in_port), Some(out_port)) = (in_port_id, out_port_id) {
            return in_port.val == out_port.val;
        }
        false
    }

    fn validate_connections(&self, connections: Vec<Vec<Connection<Match<'p, 'd>>>>) -> bool {
        for connection_set in connections {
            let mut valid = false;
            for conn in connection_set {
                if self.validate_connection(conn) {
                    valid = true;
                    break;
                }
            }
            if !valid {
                return false;
            }
        }
        true
    }
}
