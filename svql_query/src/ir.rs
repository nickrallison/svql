// New file: svql_query/src/ir.rs
// Intermediate Representation for Queries (DAG-Optimized)

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::svql_driver::Context;
use crate::svql_subgraph::GraphIndex;
use crate::traits::Component;
use crate::{Match, Search};
use ahash::{AHashMap, AHasher};
use svql_common::Config;
use svql_driver::DriverKey;
use svql_subgraph::cell::CellWrapper; // Fast hashing (add to Cargo.toml: ahash = "0.8")

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Schema {
    pub columns: Vec<String>, // e.g., ["logic.y", "reg.clk", "reg.d"]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlanNodeId(usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JoinConstraint {
    Eq((usize, usize), (usize, usize)), // (local_input_idx, col_idx)
    Or(Vec<((usize, usize), (usize, usize))>),
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

// Stub: Tree -> DAG (merge identical nodes)
pub fn canonicalize_to_dag(root_plan: LogicalPlan) -> QueryDag {
    let mut node_map: AHashMap<NodeHash, PlanNodeId> = AHashMap::new();
    let mut nodes = Vec::new();

    let root_id = canonicalize_rec(&root_plan, &mut node_map, &mut nodes);
    QueryDag {
        nodes,
        root: root_id,
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct NodeHash(u64); // Stub hash

fn node_hash(plan: &LogicalPlan) -> NodeHash {
    let mut hasher = AHasher::default();
    plan.hash(&mut hasher); // Derive Hash on LogicalPlan
    NodeHash(hasher.finish())
}

fn canonicalize_rec(
    plan: &LogicalPlan,
    node_map: &mut AHashMap<NodeHash, PlanNodeId>,
    nodes: &mut Vec<LogicalPlanNode>,
) -> PlanNodeId {
    let hash = node_hash(plan);
    if let Some(&id) = node_map.get(&hash) {
        return id;
    }

    // Recurse children
    let input_ids = match plan {
        LogicalPlan::Scan { .. } => vec![],
        LogicalPlan::Join { inputs, .. } | LogicalPlan::Union { inputs, .. } => inputs
            .iter()
            .map(|child| canonicalize_rec(child, node_map, nodes))
            .collect(),
    };

    // Rewrite constraints (local indices)
    let node = match plan {
        LogicalPlan::Scan {
            key,
            config,
            schema,
        } => LogicalPlanNode::Scan {
            key: key.clone(),
            config: config.clone(),
            schema: schema.clone(),
        },
        LogicalPlan::Join {
            constraints,
            schema,
            ..
        } => {
            let rewritten = rewrite_constraints(constraints, &input_ids);
            LogicalPlanNode::Join {
                input_ids,
                constraints: rewritten,
                schema: schema.clone(),
            }
        }
        LogicalPlan::Union {
            schema,
            tag_results,
            ..
        } => LogicalPlanNode::Union {
            input_ids,
            schema: schema.clone(),
            tag_results: *tag_results,
        },
    };

    let id = PlanNodeId(nodes.len());
    nodes.push(node);
    node_map.insert(hash, id);
    id
}

fn rewrite_constraints(
    _constraints: &[JoinConstraint],
    _input_ids: &[PlanNodeId],
) -> Vec<JoinConstraint> {
    todo!("Remap constraint indices to local input_ids")
}

// --- Flat Results (Unchanged) ---
#[derive(Clone, Debug)]
pub struct FlatResult<'a> {
    pub cells: Vec<CellWrapper<'a>>,
    pub variant_choices: Vec<usize>,
}

pub struct ExecutionResult<'a> {
    pub schema: Schema,
    pub rows: Box<dyn Iterator<Item = FlatResult<'a>> + Send + 'a>,
}

// --- Cursor (Unchanged) ---
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
    pub fn next_cell(&mut self) -> CellWrapper<'a> {
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
    fn execute_dag(&self, dag: &QueryDag, ctx: &Context) -> ExecutionResult;
}

// Stub NaiveExecutor (current logic)
pub struct NaiveExecutor;

impl Executor for NaiveExecutor {
    fn execute_dag(&self, dag: &QueryDag, _ctx: &Context) -> ExecutionResult {
        todo!("Topo-execute DAG: cache shared Scans, cross-join Joins, concat Unions")
    }
}

pub fn compute_schema_mapping(expected: &Schema, actual: &Schema) -> Vec<usize> {
    todo!("Map expected cols to actual cols")
}

// --- LogicalPlan (Tree Node, for macro gen) ---
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
