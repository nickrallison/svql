//! Intermediate Representation for Queries (DAG-Optimized)

use ahash::AHashMap;
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    let mut node_map: AHashMap<LogicalPlan, PlanNodeId> = AHashMap::new();
    let mut nodes = Vec::new();

    let root_id = canonicalize_rec(root_plan, &mut node_map, &mut nodes);
    QueryDag {
        nodes,
        root: root_id,
    }
}

fn canonicalize_rec(
    plan: LogicalPlan,
    node_map: &mut AHashMap<LogicalPlan, PlanNodeId>,
    nodes: &mut Vec<LogicalPlanNode>,
) -> PlanNodeId {
    if let Some(&id) = node_map.get(&plan) {
        return id;
    }

    let plan_key = plan.clone();

    let node = match plan {
        LogicalPlan::Scan {
            key,
            config,
            schema,
        } => LogicalPlanNode::Scan {
            key,
            config,
            schema,
        },
        LogicalPlan::Join {
            inputs,
            constraints,
            schema,
        } => {
            let input_ids = inputs
                .into_iter()
                .map(|child| canonicalize_rec(*child, node_map, nodes))
                .collect();
            LogicalPlanNode::Join {
                input_ids,
                constraints,
                schema,
            }
        }
        LogicalPlan::Union {
            inputs,
            schema,
            tag_results,
        } => {
            let input_ids = inputs
                .into_iter()
                .map(|child| canonicalize_rec(*child, node_map, nodes))
                .collect();
            LogicalPlanNode::Union {
                input_ids,
                schema,
                tag_results,
            }
        }
    };

    let id = PlanNodeId(nodes.len());
    nodes.push(node);
    node_map.insert(plan_key, id);
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
    fn execute_dag<'a>(
        &self,
        dag: &QueryDag,
        ctx: &'a Context,
        haystack_key: &DriverKey,
        config: &Config,
    ) -> ExecutionResult<'a>;
}

#[derive(Debug)]
pub struct NaiveExecutor;

impl Executor for NaiveExecutor {
    fn execute_dag<'a>(
        &self,
        dag: &QueryDag,
        ctx: &'a Context,
        haystack_key: &DriverKey,
        config: &Config,
    ) -> ExecutionResult<'a> {
        let root_result = self.execute_node(dag.root, dag, ctx, haystack_key, config);
        let schema = match dag.root_node() {
            LogicalPlanNode::Scan { schema, .. } => schema.clone(),
            LogicalPlanNode::Join { schema, .. } => schema.clone(),
            LogicalPlanNode::Union { schema, .. } => schema.clone(),
        };
        ExecutionResult {
            schema,
            rows: root_result,
        }
    }
}

impl NaiveExecutor {
    /// Recursively executes a single node in the DAG.
    fn execute_node<'a>(
        &self,
        node_id: PlanNodeId,
        dag: &QueryDag,
        ctx: &'a Context,
        haystack_key: &DriverKey,
        config: &Config,
    ) -> Vec<FlatResult<'a>> {
        match dag.node(node_id) {
            LogicalPlanNode::Scan {
                key,
                config: node_config,
                schema,
            } => {
                let needle_container = ctx.get(key).unwrap();
                let haystack_container = ctx.get(haystack_key).unwrap();
                let needle = needle_container.design();
                let haystack = haystack_container.design();
                let needle_index = needle_container.index();
                let haystack_index = haystack_container.index();

                let embeddings = ::svql_subgraph::SubgraphMatcher::enumerate_with_indices(
                    needle,
                    haystack,
                    needle_index,
                    haystack_index,
                    node_config,
                );

                embeddings
                    .items
                    .iter()
                    .map(|embedding| {
                        let cells = schema
                            .columns
                            .iter()
                            .filter_map(|wire_name| {
                                ::svql_query::traits::netlist::resolve_wire(
                                    embedding,
                                    &embeddings,
                                    needle,
                                    wire_name,
                                )
                            })
                            .collect();
                        FlatResult {
                            cells,
                            variant_choices: vec![],
                        }
                    })
                    .collect()
            }
            LogicalPlanNode::Join {
                input_ids,
                constraints,
                ..
            } => {
                let input_results: Vec<Vec<FlatResult<'a>>> = input_ids
                    .iter()
                    .map(|&id| self.execute_node(id, dag, ctx, haystack_key, config))
                    .collect();

                if input_results.is_empty() {
                    return vec![];
                }

                let combos = Self::cartesian_product(&input_results);

                combos
                    .into_iter()
                    .filter_map(|combo| {
                        let satisfied = constraints.iter().all(|constraint| match constraint {
                            JoinConstraint::Eq((i1, c1), (i2, c2)) => {
                                combo[*i1].cells[*c1] == combo[*i2].cells[*c2]
                            }
                            JoinConstraint::Or(pairs) => {
                                pairs.iter().any(|&((i1, c1), (i2, c2))| {
                                    combo[i1].cells[c1] == combo[i2].cells[c2]
                                })
                            }
                        });

                        if satisfied {
                            let cells = combo.iter().flat_map(|row| row.cells.clone()).collect();
                            let variant_choices = combo
                                .iter()
                                .flat_map(|row| row.variant_choices.clone())
                                .collect();
                            Some(FlatResult {
                                cells,
                                variant_choices,
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            LogicalPlanNode::Union {
                input_ids,
                tag_results,
                ..
            } => input_ids
                .iter()
                .enumerate()
                .flat_map(|(i, &id)| {
                    let mut results = self.execute_node(id, dag, ctx, haystack_key, config);
                    if *tag_results {
                        results
                            .iter_mut()
                            .for_each(|row| row.variant_choices.push(i));
                    }
                    results
                })
                .collect(),
        }
    }

    /// Helper to compute the Cartesian product of vectors of FlatResult.
    /// Returns a vector of vectors, where each inner vector is a combination of one FlatResult from each input vector.
    fn cartesian_product<'a>(results: &[Vec<FlatResult<'a>>]) -> Vec<Vec<FlatResult<'a>>> {
        if results.is_empty() {
            return vec![vec![]];
        }

        let (first, rest) = results.split_first().unwrap();
        let rest_product = Self::cartesian_product(rest);

        first
            .iter()
            .flat_map(|item| {
                rest_product.iter().map(move |combo| {
                    let mut new_combo = vec![item.clone()];
                    new_combo.extend(combo.clone());
                    new_combo
                })
            })
            .collect()
    }
}

pub fn compute_schema_mapping(expected: &Schema, _actual: &Schema) -> Vec<usize> {
    (0..expected.columns.len()).collect()
}
