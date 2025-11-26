//! Intermediate Representation for Queries (DAG-Optimized) - WIP

use ahash::{AHashMap, AHasher};
use svql_common::Config;
use svql_driver::{Context, DriverKey};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Schema {
    pub columns: Vec<String>, // e.g., ["logic.y", "reg.clk", "reg.d"]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlanNodeId(usize);

#[derive(Clone, Debug)]
pub enum LogicalPlanNode {
    Scan {
        key: DriverKey,
        config: Config,
        schema: Schema,
    },
    Join {
        input_ids: Vec<PlanNodeId>,
        constraints: Vec<JoinConstraint>,
        schema: Schema,
    },
    Union {
        input_ids: Vec<PlanNodeId>,
        schema: Schema,
        tag_results: bool,
    },
}

#[derive(Clone, Debug)]
pub struct QueryDag {
    nodes: Vec<LogicalPlanNode>,
    root: PlanNodeId,
}

impl QueryDag {
    pub fn node(&self, id: PlanNodeId) -> &LogicalPlanNode {
        &self.nodes[id.0]
    }
    pub fn root_node(&self) -> &LogicalPlanNode {
        self.node(self.root)
    }
}

// Stub LogicalPlan (tree node)
#[derive(Clone, Debug)]
pub enum LogicalPlan {
    Scan {
        key: DriverKey,
        config: Config,
        schema: Schema,
    },
    Join {
        inputs: Vec<LogicalPlan>,
        constraints: Vec<JoinConstraint>,
        schema: Schema,
    },
    Union {
        inputs: Vec<LogicalPlan>,
        schema: Schema,
        tag_results: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JoinConstraint {
    Eq((usize, usize), (usize, usize)), // (local_input_idx, col_idx)
    Or(Vec<((usize, usize), (usize, usize))>),
}

// Stubs (WIP)
pub fn canonicalize_to_dag(_root_plan: LogicalPlan) -> QueryDag {
    todo!("Query planner WIP: canonicalize_to_dag")
}

#[derive(Clone, PartialEq, Eq)]
struct NodeHash(u64);

fn node_hash(_plan: &LogicalPlan) -> NodeHash {
    NodeHash(42u64)
}

fn rewrite_constraints(
    _constraints: &[JoinConstraint],
    _input_ids: &[PlanNodeId],
) -> Vec<JoinConstraint> {
    todo!("Remap constraint indices")
}

// --- Flat Results ---
#[derive(Clone, Debug)]
pub struct FlatResult<'a> {
    pub cells: Vec<svql_subgraph::cell::CellWrapper<'a>>,
    pub variant_choices: Vec<usize>,
}

pub struct ExecutionResult<'a> {
    pub schema: Schema,
    pub rows: Box<dyn Iterator<Item = FlatResult<'a>> + Send + 'a>,
}

// --- Cursor ---
#[derive(Debug)]
pub struct ResultCursor<'a> {
    row: &'a FlatResult<'a>,
    mapping: &'a [usize],
    logical_ptr: usize,
    variant_ptr: usize,
}

impl<'a> ResultCursor<'a> {
    pub fn new(row: &'a FlatResult<'a>, mapping: &'a [usize]) -> Self {
        Self {
            row,
            mapping,
            logical_ptr: 0,
            variant_ptr: 0,
        }
    }
    pub fn next_cell(&mut self) -> svql_subgraph::cell::CellWrapper<'a> {
        let idx = self.mapping[self.logical_ptr];
        self.logical_ptr += 1;
        self.row.cells[idx].clone()
    }
    pub fn next_variant(&mut self) -> usize {
        let v = self.row.variant_choices[self.variant_ptr];
        self.variant_ptr += 1;
        v
    }
}

// --- Executors ---
pub trait Executor {
    fn execute_dag(&self, _dag: &QueryDag, _ctx: &Context) -> ExecutionResult<'_>;
}

#[derive(Debug)]
pub struct NaiveExecutor;

impl Executor for NaiveExecutor {
    fn execute_dag(&self, _dag: &QueryDag, _ctx: &Context) -> ExecutionResult<'_> {
        todo!("NaiveExecutor WIP")
    }
}

pub fn compute_schema_mapping(_expected: &Schema, _actual: &Schema) -> Vec<usize> {
    todo!("Schema mapping WIP")
}
