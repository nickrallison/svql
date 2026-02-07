// svql_query/src/session/connectivity_index.rs

use crate::prelude::*;


/// Maps relationships between two pattern tables based on a connection.
///
/// For `connection(from = ["a", "y"], to = ["b", "x"])`:
/// - `forward[a_row]` = set of B rows where `a.y` drives `b.x`
/// - `reverse[b_row]` = set of A rows that can drive `b.x`
pub struct BipartiteIndex {
    /// Maps source row index → set of valid target row indices
    pub forward: HashMap<u32, HashSet<u32>>,

    /// Maps target row index → set of valid source row indices
    pub reverse: HashMap<u32, HashSet<u32>>,

    /// Total number of valid edges in the bipartite graph
    pub edge_count: usize,
}

impl BipartiteIndex {
    /// Build the index for a single connection between two tables.
    ///
    /// # Performance
    /// - Time: O(|A| × |B|) connectivity checks (done once)
    /// - Space: O(valid_edges) (sparse for typical hardware graphs)
    pub fn build(
        a_table: &dyn AnyTable,
        b_table: &dyn AnyTable,
        connection: &Connection,
        ctx: &ExecutionContext,
    ) -> Self {
        let mut forward: HashMap<u32, HashSet<u32>> = HashMap::new();
        let mut reverse: HashMap<u32, HashSet<u32>> = HashMap::new();
        let mut edge_count = 0;

        // Iterate all (A, B) pairs - this is the O(|A| × |B|) work
        for a_idx in 0..a_table.len() as u32 {
            for b_idx in 0..b_table.len() as u32 {
                // Resolve both endpoints relative to the submodule
                let a_cell =
                    a_table.resolve_path(a_idx as usize, connection.from.selector.tail(), ctx);
                let b_cell =
                    b_table.resolve_path(b_idx as usize, connection.to.selector.tail(), ctx);

                // Check physical connectivity
                match (&a_cell, &b_cell) {
                    (Some(src), Some(dst)) if src.as_u64() == dst.as_u64() => {
                        forward.entry(a_idx).or_default().insert(b_idx);
                        reverse.entry(b_idx).or_default().insert(a_idx);
                        edge_count += 1;
                    }
                    _ => {}
                }
            }
        }

        tracing::debug!(
            "Built bipartite index: {}×{} → {} edges ({:.1}% density)",
            a_table.len(),
            b_table.len(),
            edge_count,
            100.0 * edge_count as f64 / (a_table.len() * b_table.len()) as f64
        );

        Self {
            forward,
            reverse,
            edge_count,
        }
    }

    /// Get all valid target rows for a given source row.
    #[inline]
    pub fn targets(&self, source_row: u32) -> Option<&HashSet<u32>> {
        self.forward.get(&source_row)
    }

    /// Get all valid source rows for a given target row.
    #[inline]
    pub fn sources(&self, target_row: u32) -> Option<&HashSet<u32>> {
        self.reverse.get(&target_row)
    }

    /// Check if a specific (source, target) pair is valid.
    #[inline]
    pub fn is_valid(&self, source_row: u32, target_row: u32) -> bool {
        self.forward
            .get(&source_row)
            .is_some_and(|targets| targets.contains(&target_row))
    }

    /// Merge another index into this one (Union of edges).
    pub fn merge(&mut self, other: Self) {
        for (src, targets) in other.forward {
            self.forward.entry(src).or_default().extend(targets);
        }
        for (target, sources) in other.reverse {
            self.reverse.entry(target).or_default().extend(sources);
        }
        self.edge_count = self.forward.values().map(|s| s.len()).sum();
    }
}
