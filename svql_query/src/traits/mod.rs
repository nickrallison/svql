//! Core traits for query components and execution.
//!
//! Defines the interfaces that all netlists, composites, and variants must
//! implement to participate in the SVQL query system.

pub mod composite;
pub mod netlist;
pub mod variant;

use std::sync::Arc;

use crate::prelude::*;

/// Links the Search and Match phases of a query component.
pub trait Projected {
    /// The Search (Pattern) version of this component.
    type Pattern: Searchable;

    /// The Match (Result) version of this component.
    type Result<'a>: Component<Match>;
}

/// Base trait for all query components.
pub trait Component<S: State> {
    /// Returns the hierarchical path of the component.
    fn path(&self) -> &Instance;

    /// Returns the static type name of the component.
    fn type_name(&self) -> &'static str;

    /// Finds a port wire by its absolute hierarchical path.
    fn find_port(&self, path: &Instance) -> Option<&Wire<S>>;

    /// Finds a port wire by its relative path segments.
    fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>>;

    /// Returns a formatted label for logging.
    fn log_label(&self) -> String {
        format!("[{} @ {}]", self.type_name(), self.path().inst_path())
    }
}

/// Trait for components that can generate a hierarchical report.
pub trait Reportable {
    /// Converts the component match into a report node.
    fn to_report(&self, name: &str) -> ReportNode;
}

/// Trait for components that can be instantiated in the Search state.
pub trait Searchable: Sized + Component<Search> {
    /// Creates a new instance of the component at the given path.
    fn instantiate(base_path: Instance) -> Self;
}

/// Interface for executing queries using the legacy backtracking engine.
pub trait Query: Component<Search> + Searchable {
    /// The type returned when a match is found.
    type Matched<'a>: Component<Match>;

    /// Executes the query against a design context.
    fn query<'a>(
        &self,
        driver: &Driver,
        context: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>>;
}

// /// Interface for executing queries using the optimized IR planner.
// pub trait PlannedQuery: Query {
//     /// Generates the logical plan tree for this query.
//     fn to_ir(&self, _config: &Config) -> LogicalPlan {
//         panic!("PlannedQuery::to_ir not implemented for this component")
//     }

//     /// Generates a canonical DAG from the logical plan.
//     fn dag_ir(&self, config: &Config) -> QueryDag {
//         crate::ir::canonicalize_to_dag(self.to_ir(config))
//     }

//     /// Reconstructs a structured match from a flat result cursor.
//     fn reconstruct<'a>(&self, _cursor: &mut ResultCursor<'a>) -> Self::Matched<'a> {
//         panic!("PlannedQuery::reconstruct not implemented for this component")
//     }

//     /// Maps a relative path to a schema column index.
//     fn get_column_index(&self, _rel_path: &[Arc<str>]) -> Option<usize> {
//         None
//     }

//     /// Returns the expected output schema for this query.
//     fn expected_schema(&self) -> Schema {
//         Schema { columns: vec![] }
//     }

//     /// Executes the query using the provided planner and executor.
//     fn query_planned<'a, 'b, T: Executor>(
//         &self,
//         executor: &'b T,
//         ctx: &'a Context,
//         key: &DriverKey,
//         config: &Config,
//     ) -> Vec<Self::Matched<'a>>
//     where
//         'b: 'a,
//     {
//         let dag = self.dag_ir(config);
//         let exec_res = executor.execute_dag(&dag, ctx, key, config);
//         let expected = self.expected_schema();
//         let mapping = compute_schema_mapping(&expected, &exec_res.schema);

//         let mut results = Vec::new();
//         for row in exec_res.rows {
//             let mut cursor = ResultCursor::new(row, mapping.clone());
//             results.push(self.reconstruct(&mut cursor));
//         }
//         results
//     }
// }

/// Validates that a physical connection exists between two matched wires in the haystack.
pub fn validate_connection<'ctx>(
    from: &Wire<Match>,
    to: &Wire<Match>,
    haystack_index: &GraphIndex<'ctx>,
) -> bool {
    validate_connection_inner(from, to, haystack_index).unwrap_or(false)
}

/// Private helper to resolve CellInfo to CellWrappers and check connectivity.
fn validate_connection_inner<'ctx>(
    from: &Wire<Match>,
    to: &Wire<Match>,
    haystack_index: &GraphIndex<'ctx>,
) -> Option<bool> {
    let from_id = from.inner.as_ref()?.id;
    let to_id = to.inner.as_ref()?.id;

    let f_wrapper = haystack_index.get_cell_by_id(from_id)?;
    let t_wrapper = haystack_index.get_cell_by_id(to_id)?;

    let fanout = haystack_index.fanout_set(&f_wrapper)?;
    Some(fanout.contains(&t_wrapper))
}
