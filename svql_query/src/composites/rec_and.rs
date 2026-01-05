use common::{Config, ModuleConfig};
use driver::{Context, Driver, DriverKey};
use std::sync::Arc;
use subgraph::GraphIndex;

use crate::prelude::*;

// #[recursive_composite]
// pub struct RecAnd<S: State> {
//     #[path]
//     pub path: Instance,
//     #[submodule]
//     pub and: AndGate<S>,
//     #[recursive_submodule]
//     pub child: Option<Box<Self>>,
// }

#[derive(Debug, Clone)]
pub struct RecAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub and: AndGate<S>,
    pub child: Option<Box<Self>>,
}

impl<S> RecAnd<S>
where
    S: State,
{
    /// Get the depth of this recursive structure (1 for just an AND gate, 2+ for nested)
    pub fn depth(&self) -> usize {
        1 + self.child.as_ref().map(|c| c.depth()).unwrap_or(0)
    }

    /// Get the output wire of the top-level AND gate
    pub fn output(&self) -> &Wire<S> {
        &self.and.y
    }
}

impl Projected for RecAnd<Search> {
    type Pattern = RecAnd<Search>;
    type Result = RecAnd<Match>;
}

impl Projected for RecAnd<Match> {
    type Pattern = RecAnd<Search>;
    type Result = RecAnd<Match>;
}

impl<S> Component<S> for RecAnd<S>
where
    S: State,
{
    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "RecAnd"
    }

    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        if let Some(port) = self.and.find_port(p) {
            return Some(port);
        }

        if let Some(ref child) = self.child {
            if let Some(port) = child.find_port(p) {
                return Some(port);
            }
        }

        None
    }

    fn find_port_inner(&self, _rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        None
    }
}

impl<S> Topology<S> for RecAnd<S>
where
    S: State,
{
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        if let Some(ref child) = self.child {
            ctx.connect_any(&[
                (Some(&child.and.y), Some(&self.and.a)),
                (Some(&child.and.y), Some(&self.and.b)),
            ]);
        }
    }
}

impl Searchable for RecAnd<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self::new(base_path)
    }
}

impl<'a> crate::traits::Reportable for RecAnd<Match> {
    fn to_report(&self, name: &str) -> crate::report::ReportNode {
        let mut children = Vec::new();
        let mut current = self.child.as_ref();
        let mut idx = 0;

        while let Some(child) = current {
            children.push(child.and.to_report(&format!("[{}]", idx)));
            current = child.child.as_ref();
            idx += 1;
        }

        crate::report::ReportNode {
            name: name.to_string(),
            type_name: "RecAnd".to_string(),
            path: self.path.clone(),
            details: Some(format!("Depth: {}", self.depth())),
            source_loc: Some(
                self.and
                    .y
                    .inner
                    .as_ref()
                    .and_then(|c| c.get_source())
                    .unwrap_or_else(|| subgraph::cell::SourceLocation {
                        file: std::sync::Arc::from(""),
                        lines: Vec::new(),
                    }),
            ),
            children,
        }
    }
}

impl RecAnd<Search> {
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            and: AndGate::instantiate(path.child("and")),
            child: None,
        }
    }
}

impl Query for RecAnd<Search> {
    fn query(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Result> {
        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::query: starting recursive AND gate search"
        );

        let haystack_index = context.get(key).unwrap().index();

        let and_query = AndGate::<Search>::instantiate(self.path.child("and"));
        let all_and_gates = and_query.query(driver, context, key, config);

        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::query: Found {} total AND gates in design",
            all_and_gates.len()
        );

        let mut current_layer: Vec<RecAnd<Match>> = all_and_gates
            .iter()
            .map(|and_gate| RecAnd {
                path: self.path.clone(),
                and: and_gate.clone(),
                child: None,
            })
            .collect();

        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::query: Layer 1 (base case) has {} matches",
            current_layer.len()
        );

        let mut all_results = current_layer.clone();
        let mut layer_num = 2;

        loop {
            let next_layer =
                build_next_layer(&self.path, &all_and_gates, &current_layer, haystack_index);

            if next_layer.is_empty() {
                tracing::event!(
                    tracing::Level::INFO,
                    "RecAnd::query: No more matches at layer {}, stopping",
                    layer_num
                );
                break;
            }

            tracing::event!(
                tracing::Level::INFO,
                "RecAnd::query: Layer {} has {} matches",
                layer_num,
                next_layer.len()
            );

            all_results.extend(next_layer.iter().cloned());
            current_layer = next_layer;
            layer_num += 1;

            if let Some(max) = config.max_recursion_depth {
                if layer_num > max {
                    tracing::event!(
                        tracing::Level::INFO,
                        "RecAnd::query: Reached max recursion depth of {}, stopping",
                        max
                    );
                    break;
                }
            }
        }

        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::query: Total {} matches across {} layers",
            all_results.len(),
            layer_num - 1
        );

        all_results
    }

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        AndGate::<Search>::context(driver, config)
    }
}

fn rec_and_cell_ids(rec_and: &RecAnd<Match>) -> Vec<usize> {
    let mut ids = Vec::new();
    if let Some(ref info) = rec_and.and.y.inner {
        ids.push(info.id);
    }

    if let Some(ref child) = rec_and.child {
        ids.extend(rec_and_cell_ids(child));
    }

    ids
}

fn build_next_layer<'ctx>(
    path: &Instance,
    all_and_gates: &[AndGate<Match>],
    prev_layer: &[RecAnd<Match>],
    haystack_index: &GraphIndex<'ctx>,
) -> Vec<RecAnd<Match>> {
    let start_time = std::time::Instant::now();
    let mut next_layer = Vec::new();
    let mut candidates_checked = 0;
    let mut validations_passed = 0;

    for prev in prev_layer {
        let Some(top_info) = &prev.and.y.inner else {
            continue;
        };
        let Some(top_and_wrapper) = haystack_index.get_cell_by_id(top_info.id) else {
            continue;
        };

        let fanout = haystack_index
            .fanout_set(&top_and_wrapper)
            .expect("Fanout not found for cell");

        let contained_ids = rec_and_cell_ids(prev);

        for and_gate in all_and_gates {
            let Some(gate_info) = &and_gate.y.inner else {
                continue;
            };
            let Some(gate_wrapper) = haystack_index.get_cell_by_id(gate_info.id) else {
                continue;
            };

            if !fanout.contains(&gate_wrapper) || contained_ids.contains(&gate_info.id) {
                continue;
            }

            candidates_checked += 1;

            let mut child = prev.clone();
            update_rec_and_path(&mut child, path.child("child"));

            let candidate = RecAnd {
                path: path.clone(),
                and: and_gate.clone(),
                child: Some(Box::new(child)),
            };

            let mut builder = ConnectionBuilder {
                constraints: Vec::new(),
            };
            candidate.define_connections(&mut builder);

            let mut valid = true;
            for group in builder.constraints {
                let mut group_satisfied = false;
                for (from, to) in group {
                    if let (Some(f), Some(t)) = (from, to) {
                        if validate_connection(f, t, haystack_index) {
                            group_satisfied = true;
                            break;
                        }
                    }
                }
                if !group_satisfied {
                    valid = false;
                    break;
                }
            }

            if valid {
                validations_passed += 1;
                next_layer.push(candidate);
            }
        }
    }

    let total_duration = start_time.elapsed();
    tracing::event!(
        tracing::Level::INFO,
        "build_next_layer: Completed in {:?}, checked {} candidates, {} passed",
        total_duration,
        candidates_checked,
        validations_passed
    );

    next_layer
}

fn update_rec_and_path<'ctx>(rec_and: &mut RecAnd<Match>, new_path: Instance) {
    rec_and.path = new_path.clone();
    let and_path = new_path.child("and");
    rec_and.and.path = and_path.clone();
    rec_and.and.a.path = and_path.child("a");
    rec_and.and.b.path = and_path.child("b");
    rec_and.and.y.path = and_path.child("y");

    if let Some(ref mut child) = rec_and.child {
        update_rec_and_path(child, new_path.child("child"));
    }
}
