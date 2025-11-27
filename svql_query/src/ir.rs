//! Intermediate Representation for Queries (DAG-Optimized)

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
        needle_key: DriverKey,
        haystack_key: DriverKey, // FIXED: Track both
        config_hash: u64,
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
    // NEW: Filter for Topology post-join
    Filter {
        input_id: PlanNodeId,
        schema: Schema,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JoinConstraint {
    Eq((usize, usize), (usize, usize)), // (left.col, right.col)
    Or(Vec<((usize, usize), (usize, usize))>),
}

#[derive(Clone, Debug)]
pub enum LogicalPlan {
    Scan {
        needle_key: DriverKey,
        haystack_key: DriverKey,
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
    Filter {
        input: Box<LogicalPlan>,
        schema: Schema,
    },
}

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
pub struct FlatResult<'a> {
    pub cells: Vec<CellWrapper<'a>>,
    pub variant_choices: Vec<usize>, // NEW: For variants
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
    fn execute_dag(&self, dag: &QueryDag, ctx: &Context) -> ExecutionResult<'_>;
}

// NEW: NaiveExecutor impl below (piece 3)
pub struct NaiveExecutor;

#[derive(Clone, PartialEq, Eq, Hash)]
struct NodeHash(u64);

fn node_hash(plan: &LogicalPlan) -> NodeHash {
    let mut hasher = AHasher::default();
    std::mem::discriminant(plan).hash(&mut hasher);
    // FIXED: Hash fields recursively (stub: children/config)
    match plan {
        LogicalPlan::Scan {
            needle_key, config, ..
        } => {
            needle_key.hash(&mut hasher);
            // Config hash stub
            42u64.hash(&mut hasher);
        }
        _ => {} // Expand for Join/Union
    }
    NodeHash(hasher.finish())
}

pub fn canonicalize_to_dag(mut root_plan: LogicalPlan) -> QueryDag {
    let mut node_map: AHashMap<NodeHash, PlanNodeId> = AHashMap::new();
    let mut nodes = Vec::new();

    let root_id = canonicalize_rec(&mut root_plan, &mut node_map, &mut nodes);
    QueryDag {
        nodes,
        root: root_id,
    }
}

fn canonicalize_rec(
    plan: &mut LogicalPlan,
    node_map: &mut AHashMap<NodeHash, PlanNodeId>,
    nodes: &mut Vec<LogicalPlanNode>,
) -> PlanNodeId {
    let hash = node_hash(plan);
    if let Some(&id) = node_map.get(&hash) {
        return id;
    }

    // FIXED: Mut recurse + dedup children
    let input_ids = match plan {
        LogicalPlan::Scan {
            needle_key,
            haystack_key,
            config,
            schema,
        } => {
            let config_hash = {
                let mut h = AHasher::default();
                // Stub: Proper config hash
                config.hash(&mut h);
                h.finish()
            };
            let node = LogicalPlanNode::Scan {
                needle_key: needle_key.clone(),
                haystack_key: haystack_key.clone(),
                config_hash,
                schema: schema.clone(),
            };
            let id = PlanNodeId(nodes.len());
            nodes.push(node);
            node_map.insert(hash, id);
            return id;
        }
        LogicalPlan::Join {
            inputs,
            constraints,
            schema,
        } => {
            let input_ids: Vec<_> = inputs
                .iter_mut()
                .map(|child| canonicalize_rec(child, node_map, nodes))
                .collect();
            let node = LogicalPlanNode::Join {
                input_ids,
                constraints: constraints.clone(),
                schema: schema.clone(),
            };
            let id = PlanNodeId(nodes.len());
            nodes.push(node);
            node_map.insert(hash, id);
            return id;
        }
        LogicalPlan::Union {
            inputs,
            schema,
            tag_results,
        } => {
            let input_ids: Vec<_> = inputs
                .iter_mut()
                .map(|child| canonicalize_rec(child, node_map, nodes))
                .collect();
            let node = LogicalPlanNode::Union {
                input_ids,
                schema: schema.clone(),
                tag_results: *tag_results,
            };
            let id = PlanNodeId(nodes.len());
            nodes.push(node);
            node_map.insert(hash, id);
            return id;
        }
        LogicalPlan::Filter { input, schema } => {
            let input_id = canonicalize_rec(input, node_map, nodes);
            let node = LogicalPlanNode::Filter {
                input_id,
                schema: schema.clone(),
            };
            let id = PlanNodeId(nodes.len());
            nodes.push(node);
            node_map.insert(hash, id);
            return id;
        }
    };
    // Fallback (unreachable)
    PlanNodeId(0)
}

pub fn compute_schema_mapping(expected: &Schema, actual: &Schema) -> Vec<usize> {
    // Stub: Name-based match
    expected
        .columns
        .iter()
        .map(|col| actual.columns.iter().position(|c| c == col).unwrap_or(0))
        .collect()
}
