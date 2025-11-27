pub mod composite;
pub mod netlist;
pub mod variant;

use std::sync::Arc;
use svql_common::Config;
use svql_driver::{Context, DriverKey};

use crate::ir::{
    Executor, LogicalPlan, QueryDag, ResultCursor, Schema, canonicalize_to_dag,
    compute_schema_mapping,
};
use crate::{Instance, Match, Search, State, Wire};

/// Base trait for all query components (Netlists, Composites, Variants, Wires).
pub trait Component<S: State> {
    fn path(&self) -> &Instance;
    fn type_name(&self) -> &'static str;
    fn children(&self) -> Vec<&dyn Component<S>>;

    fn find_port(&self, path: &Instance) -> Option<&Wire<S>>;
    fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>>;
}

/// Trait for components that can be instantiated in the Search state.
pub trait Searchable: Sized {
    fn instantiate(base_path: Instance) -> Self;
}

/// Legacy Query trait (planner WIP: new methods to be added later).
pub trait Query: Component<Search> {
    type Matched<'a>: Component<Match<'a>>;

    fn query<'a>(
        &self,
        driver: &::svql_driver::Driver,
        context: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>>;
}

/// PlannedQuery supertrait: Enables optimized planner execution.
/// Implement alongside `Query` for caching/DAG benefits.
/// Defaults use legacy `Query::query` as fallback.
pub trait PlannedQuery<'q>: Query {
    /// Generate LogicalPlan tree for this query.
    fn to_ir(&self, config: &Config) -> LogicalPlan {
        // Default: Panic → encourage impl for planner users.
        // FUTURE: Auto-gen from structure via macros/reflection.
        panic!(
            "PlannedQuery::to_ir must be implemented for planner execution (or use legacy Query::query)"
        )
    }

    /// Canonical DAG (shared subplans).
    fn dag_ir(&self, config: &Config) -> QueryDag {
        canonicalize_to_dag(self.to_ir(config))
    }

    /// Reconstruct Matched from flat execution result cursor.
    fn reconstruct<'a>(&self, cursor: &mut ResultCursor<'a>) -> Self::Matched<'a> {
        panic!("PlannedQuery::reconstruct must be implemented")
    }

    /// Map relative path (e.g., ["logic", "y"]) to schema column index.
    fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize> {
        // Default: Walk structure recursively.
        // Stub: Requires schema knowledge.
        None
    }

    /// Expected output schema (column paths).
    fn expected_schema(&self) -> Schema {
        Schema { columns: vec![] }
    }

    /// Planner-optimized query: Uses DAG + Executor (caching/indexes).
    fn query_planned<'a, T: Executor>(
        &self,
        executor: &'a T,
        ctx: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>> {
        let dag = self.dag_ir(config);
        let exec_res = executor.execute_dag(&dag, ctx);
        let expected = self.expected_schema();
        let mapping = compute_schema_mapping(&expected, &exec_res.schema);
        exec_res
            .rows
            .into_iter()
            .map(move |row| {
                let mut cursor = ResultCursor::new(&row, &mapping);
                self.reconstruct(&mut cursor)
            })
            .collect()
    }
}

// --- Topology & Connections ---
pub use crate::traits::composite::ConnectionBuilder;
pub use crate::traits::composite::Topology;

/// Validate if `from` drives `to` in haystack.
pub fn validate_connection<'ctx>(
    from: &Wire<Match<'ctx>>,
    to: &Wire<Match<'ctx>>,
    haystack_index: &svql_subgraph::GraphIndex<'ctx>,
) -> bool {
    let from_cell = &from.inner;
    let to_cell = &to.inner;

    haystack_index
        .fanout_set(from_cell)
        .map_or(false, |set| set.contains(to_cell))
}
