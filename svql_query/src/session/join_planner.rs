// svql_query/src/session/join_planner.rs

use crate::prelude::*;
use crate::session::PatternSchema;
use crate::session::connectivity_index::BipartiteIndex;
use crate::traits::composite::Endpoint;


/// Precomputed connectivity indices for all connections in a pattern.
pub struct ConnectivityCache {
    /// Keyed by (from_submodule_idx, to_submodule_idx, connection_idx_in_cnf)
    pub indices: HashMap<(usize, usize, usize), BipartiteIndex>,
}

impl ConnectivityCache {
    /// Build all connectivity indices for a composite pattern.
    ///
    /// # Arguments
    /// - `submodules`: Schema metadata for all submodules
    /// - `connections`: The CNF connection constraints
    /// - `dep_tables`: Dependency tables (aligned with submodules)
    /// - `ctx`: Execution context for graph access
    pub fn build<T: Composite>(
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
        ctx: &ExecutionContext,
    ) -> Self {
        let schema = T::composite_schema();
        let mut indices: HashMap<(usize, usize, usize), BipartiteIndex> = HashMap::new();

        tracing::info!(
            "[CONNECTIVITY_CACHE] Building indices for {} CNF groups",
            T::CONNECTIONS.connections.len()
        );

        // Iterate CNF groups (AND)
        for (cnf_idx, cnf_group) in T::CONNECTIONS.connections.iter().enumerate() {
            // Iterate OR alternatives
            for conn in cnf_group.iter() {
                // Determine which submodules are involved
                let from_sub = Self::resolve_submodule(&conn.from, schema);
                let to_sub = Self::resolve_submodule(&conn.to, schema);

                if let (Some(a_idx), Some(b_idx)) = (from_sub, to_sub) {
                    let key = (a_idx, b_idx, cnf_idx);
                    let index =
                        BipartiteIndex::build(dep_tables[a_idx], dep_tables[b_idx], conn, ctx);

                    if let Some(existing) = indices.get_mut(&key) {
                        existing.merge(index);
                    } else {
                        indices.insert(key, index);
                    }
                }
            }
        }

        tracing::info!(
            "[CONNECTIVITY_CACHE] Built {} unique indices",
            indices.len()
        );

        Self { indices }
    }

    /// Get the precomputed index for a specific connection.
    pub fn get(&self, from_sub: usize, to_sub: usize, cnf_idx: usize) -> Option<&BipartiteIndex> {
        self.indices.get(&(from_sub, to_sub, cnf_idx))
    }

    /// Resolve a selector path to its root submodule index.
    fn resolve_submodule(selector: &Endpoint, schema: &PatternSchema) -> Option<usize> {
        let head = selector.selector.head()?;
        schema.submodule_index(head)
    }
}
