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
    ir::{Executor, LogicalPlan, QueryDag, ResultCursor, Schema, compute_schema_mapping},
};

/// Base trait for all query components
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

/// PlannedQuery trait: Enables optimized planner execution.
pub trait PlannedQuery: Query {
    /// Generate LogicalPlan tree for this query.
    fn to_ir(&self, _config: &Config) -> LogicalPlan {
        todo!("PlannedQuery::to_ir: Generate plan tree (e.g., Scan/Join from structure)")
    }

    /// Canonical DAG (shared subplans).
    fn dag_ir(&self, config: &Config) -> QueryDag {
        crate::ir::canonicalize_to_dag(self.to_ir(config))
    }

    /// Reconstruct Matched from flat execution result cursor.
    fn reconstruct<'a>(&self, _cursor: &mut ResultCursor<'a>) -> Self::Matched<'a> {
        todo!("PlannedQuery::reconstruct: Build Matched from cursor cells/variants")
    }

    /// Map relative path (e.g., ["logic", "y"]) to schema column index.
    fn get_column_index(&self, _rel_path: &[Arc<str>]) -> Option<usize> {
        None
    }

    /// Expected output schema (column paths).
    fn expected_schema(&self) -> Schema {
        Schema { columns: vec![] }
    }

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
        let dag = self.dag_ir(config);
        let exec_res = executor.execute_dag(&dag, ctx, key, config);
        let expected = self.expected_schema();
        let mapping = compute_schema_mapping(&expected, &exec_res.schema);

        let mut results = Vec::new();
        for row in exec_res.rows {
            let mut cursor = ResultCursor::new(row, mapping.clone());
            results.push(self.reconstruct(&mut cursor));
        }
        results
    }
}

pub use composite::{ConnectionBuilder, Topology};
pub use netlist::{NetlistMeta, PortDir, PortSpec, resolve_wire};

/// Validate connection in haystack
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
