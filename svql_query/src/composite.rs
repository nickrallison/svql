use crate::haystack::HaystackPool;
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
    fn connections(&self) -> Vec<Vec<Connection<S>>>;
}

/// Search-time surface for composites.
/// - Hit<'p,'d> fixes the payload type returned for a query
/// - The query is executed under a shared HaystackPool, which owns child
///   QueryCtx instances and returns &'ctx QueryCtx on demand.
pub trait SearchableComposite: Composite<Search> {
    type Hit<'p, 'd>;

    fn query<'ctx>(
        hay: &'ctx HaystackPool,
        path: crate::instance::Instance,
        config: &svql_subgraph::config::Config,
    ) -> Vec<Self::Hit<'ctx, 'ctx>>;
}

/// Matched/instantiated composite payload, with validation helpers.
///
/// A macro can emit other_filters() to add extra user constraints. The default
/// validate_connection() checks that the Match on the "from" wire equals the Match
/// on the "to" wire.
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
        connections
            .into_iter()
            .all(|alts| alts.into_iter().any(|conn| self.validate_connection(conn)))
    }
}
