pub mod composite;
pub mod netlist;
pub mod variant;

use std::sync::Arc;

use svql_common::Config;
use svql_driver::{Context, DriverKey};
use svql_subgraph::GraphIndex;

use crate::{
    Match, Search, State, Wire,
    instance::Instance,
    ir::{
        Executor, LogicalPlan, LogicalPlanNode, QueryDag, ResultCursor, Schema,
        canonicalize_to_dag, compute_schema_mapping,
    },
};

/// Base trait for all query components (Netlists, Composites, Variants, Wires).
pub trait Component<S: State> {
    fn path(&self) -> &Instance;
    fn type_name(&self) -> &'static str;
    fn children(&self) -> Vec<&dyn Component<S>>;

    fn find_port(&self, path: &Instance) -> Option<&Wire<S>>;
    fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>>;
}

/// Trait for components that can be instantiated in the Search state.
pub trait Searchable: Sized + Component<Search> {
    fn instantiate(base_path: Instance) -> Self;
}

/// Legacy Query trait: Compatible with existing macros/manual impls.
/// Use for `query(driver, ...)`.
pub trait Query: Component<Search> + Searchable {
    type Matched<'a>: Component<Match<'a>>;

    /// Legacy query dispatcher (current implementation).
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
/// Defaults use legacy `Query::query` as fallback (TODO: bridge).
pub trait PlannedQuery: Query {
    /// Generate LogicalPlan tree for this query.
    fn to_ir(&self, _config: &Config) -> LogicalPlan;

    /// Canonical DAG (shared subplans).
    fn dag_ir(&self, config: &Config) -> QueryDag {
        canonicalize_to_dag(self.to_ir(config))
    }

    /// Reconstruct Matched from flat execution result cursor.
    fn reconstruct<'a>(&self, cursor: ResultCursor<'a>) -> Self::Matched<'a>;

    /// Map relative path (e.g., ["logic", "y"]) to schema column index.
    fn get_column_index(&self, _rel_path: &[Arc<str>]) -> Option<usize>;

    /// Expected output schema (column paths).
    fn expected_schema(&self) -> Schema;

    fn query_planned<'a, 'b, T: Executor>(
        &self,
        executor: &'b T,
        ctx: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>>
    where
        'b: 'a,
    {
        let mut dag = self.dag_ir(config);
        // Patch haystack_key into all Scans
        patch_haystack_keys(&mut dag, key);
        let exec_res = executor.execute_dag(&dag, ctx);
        let expected = self.expected_schema();
        let mapping = compute_schema_mapping(&expected, &exec_res.schema);

        exec_res
            .rows
            .into_iter()
            .map(|row| {
                let cursor = ResultCursor::new(row, mapping.clone());
                self.reconstruct(cursor)
            })
            .collect()
    }
}

fn patch_haystack_keys(dag: &mut QueryDag, key: &DriverKey) {
    for node in &mut dag.nodes {
        if let LogicalPlanNode::Scan { haystack_key, .. } = node {
            *haystack_key = key.clone();
        }
    }
}

// Re-export for convenience
pub use composite::{ConnectionBuilder, Topology};
pub use netlist::{NetlistMeta, PortDir, PortSpec, resolve_wire};

/// Validate connection in haystack (used by legacy/composite validation).
pub fn validate_connection<'ctx>(
    from: &Wire<Match<'ctx>>,
    to: &Wire<Match<'ctx>>,
    haystack_index: &GraphIndex<'ctx>,
) -> bool {
    let from_cell = &from.inner;
    let to_cell = &to.inner;
    haystack_index
        .fanout_set(from_cell)
        .map_or(false, |set| set.contains(to_cell))
}
