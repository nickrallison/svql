use crate::ir::{
    ExecutionResult, Executor, FlatResult, LogicalPlanNode, PlanNodeId, QueryDag, ResultCursor,
    Schema,
};
use crate::svql_driver::Context;
use crate::svql_subgraph::{SubgraphMatcher, cell::CellWrapper};
use itertools::iproduct;
use svql_driver::DriverKey;
use tracing::debug;

pub struct NaiveExecutor;

impl Executor for NaiveExecutor {
    fn execute_dag(&self, dag: &QueryDag, ctx: &Context) -> ExecutionResult {
        let mut results = vec![ExecutionResult::default(); dag.nodes.len()];
        self.eval_rec(dag, ctx, dag.root, &mut results)
    }
}

impl NaiveExecutor {
    fn eval_rec<'a>(
        &self,
        dag: &QueryDag,
        ctx: &'a Context,
        id: PlanNodeId,
        results: &mut Vec<ExecutionResult<'a>>,
    ) -> ExecutionResult<'a> {
        if results[id.0].rows.is_empty() {
            // Memoized
            let node = dag.node(id);
            let res = match node {
                LogicalPlanNode::Scan {
                    needle_key,
                    haystack_key,
                    config_hash: _,
                    schema,
                } => {
                    let needle_ctx = ctx.get(needle_key).expect("Needle missing");
                    let haystack_ctx = ctx.get(haystack_key).expect("Haystack missing");
                    let embeddings = SubgraphMatcher::enumerate_with_indices(
                        needle_ctx.design(),
                        haystack_ctx.design(),
                        needle_ctx.index(),
                        haystack_ctx.index(),
                        &svql_common::Config::default(), // Resolve from hash if needed
                    );
                    let rows: Vec<FlatResult> = embeddings
                        .items
                        .iter()
                        .map(|emb| FlatResult {
                            cells: schema
                                .columns
                                .iter()
                                .map(|col| {
                                    /* resolve_wire stub: emb.resolve_port(col) */
                                    unimplemented!("Port resolution")
                                })
                                .collect(),
                            variant_choices: vec![],
                        })
                        .collect();
                    ExecutionResult {
                        schema: schema.clone(),
                        rows,
                    }
                }
                LogicalPlanNode::Join {
                    input_ids,
                    constraints,
                    schema,
                } => {
                    let child_rows: Vec<_> = input_ids
                        .iter()
                        .map(|child_id| self.eval_rec(dag, ctx, *child_id, results))
                        .collect();
                    // Naive cartesian + filter constraints
                    let mut rows = Vec::new();
                    for tuple in iproduct!(child_rows.iter().map(|r| &r.rows)) {
                        let mut cells = Vec::new();
                        let mut valid = true;
                        for (group, child_res) in constraints.iter().zip(tuple) {
                            let mut group_ok = false;
                            for ((lcol, lrow), (rcol, rrow)) in group {
                                // Pseudo-code; match JoinConstraint
                                // Cross-check cells[child_idx][lcol] == cells[child_idx+1][rcol]
                                group_ok = true; // Stub validation
                                break;
                            }
                            if !group_ok {
                                valid = false;
                                break;
                            }
                        }
                        if valid {
                            // Flatten cells across children
                            cells.extend(/* merged cells */ vec![]);
                            rows.push(FlatResult {
                                cells,
                                variant_choices: vec![],
                            });
                        }
                    }
                    ExecutionResult {
                        schema: schema.clone(),
                        rows,
                    }
                }
                LogicalPlanNode::Union {
                    input_ids,
                    schema,
                    tag_results: _,
                } => {
                    let mut rows = Vec::new();
                    for input_id in input_ids {
                        let child = self.eval_rec(dag, ctx, *input_id, results);
                        rows.extend(child.rows);
                    }
                    ExecutionResult {
                        schema: schema.clone(),
                        rows,
                    }
                }
                LogicalPlanNode::Filter { input_id, schema } => {
                    let input = self.eval_rec(dag, ctx, *input_id, results);
                    // Stub: Post-join topology filter
                    ExecutionResult {
                        schema: schema.clone(),
                        rows: input.rows,
                    }
                }
            };
            results[id.0] = res;
        }
        results[id.0].clone()
    }
}
