use svql_driver::{context::Context, driver::Driver};

use crate::{Connection, Match, Search, State, WithPath};

/// A composite is a tree of sub-queries (netlists or other composites) and a set of
/// connectivity constraints. The constraints are expressed as disjunctions of simple
/// connections:
///
/// - connections() returns Vec<Vec<Connection<S>>>
///   Each inner Vec is a set of alternative (from -> to) wires, and at least one
///   must validate (OR semantics).
/// - validate_connections() checks that each inner set contains at least one valid
///   connection, using validate_connection().
pub trait Composite<S>: WithPath<S>
where
    S: State,
{
    /// Describe connectivity constraints as OR-sets of point-to-point connections.
    /// A macro can generate a very small, uniform impl for any composite.
    fn connections(&self) -> Vec<Vec<Connection<S>>>;
}

/// Search-time surface for composites.
/// A macro can generate a strongly-typed Hit<'p, 'd> and a composite-specific
/// query function with the exact pattern arguments it needs. This trait only
/// standardizes the associated hit type so filtering/validation helpers are shared.
///
/// Note: We do NOT fix the function signature of query() here because the set of
/// child patterns varies per composite. Instead, each composite can expose an
/// inherent fn query(...) that takes the needed pattern drivers and a haystack.
pub trait SearchableComposite: Composite<Search> {
    type Hit<'p, 'd>;

    fn context(&self, driver: &Driver) -> Context {
        todo!()
    }
}

/// Matched/instantiated composite payload, with validation helpers.
///
/// A macro can emit other_filters() to add extra user constraints. The default
/// validate_connection() checks that the Match on the "from" wire equals the Match
/// on the "to" wire (i.e., they represent the same design driver/source).
pub trait MatchedComposite<'p, 'd>: Composite<Match<'p, 'd>> {
    /// Additional user filters; return predicates that must all pass.
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>>;

    /// Check if the given connection picks wires whose Match values are equal.
    fn validate_connection(&self, connection: Connection<Match<'p, 'd>>) -> bool {
        let in_port_id = self.find_port(&connection.from.path);
        let out_port_id = self.find_port(&connection.to.path);

        if let (Some(in_port), Some(out_port)) = (in_port_id, out_port_id) {
            return in_port.val == out_port.val;
        }
        false
    }

    /// For each OR-set of connections, ensure at least one is valid.
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
