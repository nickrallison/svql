//! Intermediate Representation for Queries.
//!
//! Defines the logical plan nodes and execution structures used to optimize
//! and run complex hardware queries.

use ahash::AHashMap;
use svql_common::Config;
use svql_driver::{Context, DriverKey};
use svql_subgraph::cell::CellWrapper;

/// Defines the column structure of a query result.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Schema {
    /// List of column names (usually wire paths).
    pub columns: Vec<String>,
}

/// Unique identifier for a node within a query DAG.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlanNodeId(pub usize);

/// Logical operations within a query plan.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LogicalPlanNode {
    /// Scans a design for a specific netlist pattern.
    Scan {
        key: DriverKey,
        config: Config,
        schema: Schema,
    },
    /// Joins multiple sub-queries based on connectivity constraints.
    Join {
        input_ids: Vec<PlanNodeId>,
        constraints: Vec<JoinConstraint>,
        schema: Schema,
    },
    /// Combines results from multiple query variants.
    Union {
        input_ids: Vec<PlanNodeId>,
        schema: Schema,
        tag_results: bool,
    },
}

/// A directed acyclic graph representing a query execution plan.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QueryDag {
    pub nodes: Vec<LogicalPlanNode>,
    pub root: PlanNodeId,
}

impl QueryDag {
    /// Retrieves a node by its ID.
    pub fn node(&self, id: PlanNodeId) -> &LogicalPlanNode {
        &self.nodes[id.0]
    }

    /// Returns the root node of the DAG.
    pub fn root_node(&self) -> &LogicalPlanNode {
        self.node(self.root)
    }
}

/// Recursive tree representation of a logical plan.
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

/// Constraints used to filter join results.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JoinConstraint {
    /// Requires two wires to be the same physical cell.
    Eq((usize, usize), (usize, usize)),
    /// Requires at least one of the provided pairs to be the same physical cell.
    Or(Vec<((usize, usize), (usize, usize))>),
}

/// Converts a recursive logical plan into a deduplicated DAG.
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

/// A single row of a query result in flattened format.
#[derive(Clone, Debug)]
pub struct FlatResult<'a> {
    /// Cells corresponding to the schema columns.
    pub cells: Vec<CellWrapper<'a>>,
    /// Indices of chosen variants for Union nodes.
    pub variant_choices: Vec<usize>,
}

/// The complete result set of a query execution.
pub struct ExecutionResult<'a> {
    pub schema: Schema,
    pub rows: Vec<FlatResult<'a>>,
}

/// Helper to traverse a flat result row and reconstruct structured objects.
#[derive(Debug)]
pub struct ResultCursor<'a> {
    row: FlatResult<'a>,
    mapping: Vec<usize>,
    logical_ptr: usize,
    variant_ptr: usize,
}

impl<'a> ResultCursor<'a> {
    /// Creates a new cursor for a result row.
    pub fn new(row: FlatResult<'a>, mapping: Vec<usize>) -> Self {
        Self {
            row,
            mapping,
            logical_ptr: 0,
            variant_ptr: 0,
        }
    }

    /// Retrieves the next cell from the row based on the schema mapping.
    pub fn next_cell(&mut self) -> CellWrapper<'a> {
        let idx = self.mapping[self.logical_ptr];
        self.logical_ptr += 1;
        self.row.cells[idx].clone()
    }

    /// Retrieves the next variant index from the row.
    pub fn next_variant(&mut self) -> usize {
        let v = self.row.variant_choices[self.variant_ptr];
        self.variant_ptr += 1;
        v
    }
}

/// Interface for executing query plans.
pub trait Executor {
    /// Executes a query DAG against a design context.
    fn execute_dag<'a>(
        &self,
        dag: &QueryDag,
        ctx: &'a Context,
        haystack_key: &DriverKey,
        config: &Config,
    ) -> ExecutionResult<'a>;
}

/// A simple, non-optimized query executor.
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
                let needle_container = ctx.get(key).expect("Pattern design missing from context");
                let haystack_container = ctx
                    .get(haystack_key)
                    .expect("Haystack design missing from context");

                let assignments = ::svql_subgraph::SubgraphMatcher::enumerate_with_indices(
                    needle_container.design(),
                    haystack_container.design(),
                    needle_container.index(),
                    haystack_container.index(),
                    key.module_name().to_string(),
                    haystack_key.module_name().to_string(),
                    node_config,
                );

                assignments
                    .items
                    .iter()
                    .map(|assignment| {
                        let cells = schema
                            .columns
                            .iter()
                            .filter_map(|wire_name| {
                                ::svql_query::traits::netlist::resolve_wire(
                                    assignment,
                                    &assignments,
                                    needle_container.design(),
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

    /// Computes the Cartesian product of multiple result sets.
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

/// Computes the mapping between expected schema columns and actual result indices.
pub fn compute_schema_mapping(expected: &Schema, _actual: &Schema) -> Vec<usize> {
    (0..expected.columns.len()).collect()
}
