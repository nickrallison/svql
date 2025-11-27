//! Intermediate Representation for Queries (DAG-Optimized) - WIP/Stubs

use std::hash::{Hash, Hasher};

use ahash::{AHashMap, AHasher};
use svql_common::Config;
use svql_driver::{Context, DriverKey};
use svql_subgraph::cell::CellWrapper;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Schema {
    pub columns: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlanNodeId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalPlanNode {
    Scan {
        key: DriverKey,
        config_hash: u64, // Stub for Eq/Hash
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
pub struct QueryDag {
    pub nodes: Vec<LogicalPlanNode>,
    pub root: PlanNodeId,
}

impl QueryDag {
    pub fn node(&self, id: PlanNodeId) -> &LogicalPlanNode {
        &self.nodes[id.0]
    }
    pub fn root_node(&self) -> &LogicalPlanNode {
        self.node(self.root)
    }
}

#[derive(Clone, Debug)]
pub enum LogicalPlan {
    Scan {
        key: DriverKey,
        config: Config,
        schema: Schema,
    },
    Join {
        inputs: Vec<Box<LogicalPlan>>,
        constraints: Vec<JoinConstraint>,
        schema: Schema,
    },
    Union {
        inputs: Vec<Box<LogicalPlan>>,
        schema: Schema,
        tag_results: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JoinConstraint {
    Eq((usize, usize), (usize, usize)),
    Or(Vec<((usize, usize), (usize, usize))>),
}

pub fn canonicalize_to_dag(root_plan: LogicalPlan) -> QueryDag {
    let mut node_map: AHashMap<NodeHash, PlanNodeId> = AHashMap::new();
    let mut nodes = Vec::new();

    let _root_id = canonicalize_rec(&root_plan, &mut node_map, &mut nodes);
    QueryDag {
        nodes,
        root: PlanNodeId(0),
    } // Stub root
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct NodeHash(u64);

fn node_hash(plan: &LogicalPlan) -> NodeHash {
    let mut hasher = AHasher::default();
    std::mem::discriminant(plan).hash(&mut hasher);
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

    // Recurse children first
    let input_ids = match plan {
        LogicalPlan::Scan {
            key,
            config: _,
            schema,
        } => {
            let config_hash = {
                let mut h = AHasher::default();
                // Stub: Hash discriminant or fields
                42u64.hash(&mut h);
                h.finish()
            };
            let node = LogicalPlanNode::Scan {
                key: key.clone(),
                config_hash,
                schema: schema.clone(),
            };
            let id = PlanNodeId(nodes.len());
            nodes.push(node);
            node_map.insert(hash, id);
            return id;
        }
        LogicalPlan::Join { inputs, .. } | LogicalPlan::Union { inputs, .. } => inputs
            .iter()
            .map(|child| canonicalize_rec(child, node_map, nodes))
            .collect(),
    };

    // Build node (stub)
    let node = LogicalPlanNode::Union {
        input_ids,
        schema: Schema { columns: vec![] },
        tag_results: false,
    };

    let id = PlanNodeId(nodes.len());
    nodes.push(node);
    node_map.insert(hash, id);
    id
}

#[derive(Clone, Debug)]
pub struct FlatResult<'a> {
    pub cells: Vec<CellWrapper<'a>>,
    pub variant_choices: Vec<usize>,
}

pub struct ExecutionResult<'a> {
    pub schema: Schema,
    pub rows: Vec<FlatResult<'a>>,
}

#[derive(Debug)]
pub struct ResultCursor<'a> {
    row: FlatResult<'a>,
    mapping: Vec<usize>,
    logical_ptr: usize,
    variant_ptr: usize,
}

impl<'a> ResultCursor<'a> {
    pub fn new(row: FlatResult<'a>, mapping: Vec<usize>) -> Self {
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

pub trait Executor {
    fn execute_dag(&self, _dag: &QueryDag, _ctx: &Context) -> ExecutionResult<'_>;
}

#[derive(Debug)]
pub struct NaiveExecutor;

impl Executor for NaiveExecutor {
    fn execute_dag(&self, _dag: &QueryDag, _ctx: &Context) -> ExecutionResult<'_> {
        todo!("Executor DAG execution")
    }
}

pub fn compute_schema_mapping(expected: &Schema, _actual: &Schema) -> Vec<usize> {
    (0..expected.columns.len()).collect()
}
