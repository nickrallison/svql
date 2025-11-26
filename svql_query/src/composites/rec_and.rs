use std::sync::Arc;
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::{GraphIndex, cell::CellWrapper};

use crate::{
    Match, Search, State, Wire,
    instance::Instance,
    primitives::and::AndGate,
    traits::{Component, ConnectionBuilder, Query, Searchable, Topology, validate_connection},
};

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
    pub fn with_child(path: Instance, and: AndGate<S>, child: Self) -> Self {
        Self {
            path,
            and,
            child: Some(Box::new(child)),
        }
    }

    /// Get the depth of this recursive structure (1 for just an AND gate, 2+ for nested)
    pub fn depth(&self) -> usize {
        1 + self.child.as_ref().map(|c| c.depth()).unwrap_or(0)
    }

    /// Get the output wire of the top-level AND gate
    pub fn output(&self) -> &Wire<S> {
        &self.and.y
    }
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

    fn children(&self) -> Vec<&dyn Component<S>> {
        let mut kids: Vec<&dyn Component<S>> = vec![&self.and];
        if let Some(c) = &self.child {
            kids.push(c.as_ref());
        }
        kids
    }

    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        // First, check if the path is for our AND gate
        if let Some(port) = self.and.find_port(p) {
            return Some(port);
        }

        // Otherwise, check if it's for our child
        if let Some(ref child) = self.child {
            if let Some(port) = child.find_port(p) {
                return Some(port);
            }
        }

        None
    }

    fn find_port_inner(&self, _rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        // Manual composites usually delegate via find_port,
        // but if accessed via relative path, we'd need logic here.
        // For now, returning None is safe as long as find_port handles absolute paths.
        None
    }
}

impl<S> Topology<S> for RecAnd<S>
where
    S: State,
{
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        if let Some(ref child) = self.child {
            // Connection from child's output to this and's input (either a or b)
            ctx.connect_any(&[
                (Some(&child.and.y), Some(&self.and.a)),
                (Some(&child.and.y), Some(&self.and.b)),
            ]);
        }
    }
}

impl Searchable for RecAnd<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self {
            path: base_path.clone(),
            and: AndGate::new(base_path.child("and")),
            child: None,
        }
    }
}

impl RecAnd<Search> {
    pub fn new(path: Instance) -> Self {
        <Self as Searchable>::instantiate(path)
    }

    pub fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Only need context for AND gates
        AndGate::<Search>::context(driver, config)
    }
}

impl Query for RecAnd<Search> {
    type Matched<'a> = RecAnd<Match<'a>>;

    fn query<'a>(
        &self,
        driver: &Driver,
        context: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>> {
        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::query: starting recursive AND gate search"
        );

        let haystack_index = context.get(key).unwrap().index();

        // Query all AND gates once (reuse for all layers)
        // We instantiate a temporary AndGate to run the query
        let and_query = AndGate::<Search>::instantiate(self.path.child("and"));
        let all_and_gates = and_query.query(driver, context, key, config);

        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::query: Found {} total AND gates in design",
            all_and_gates.len()
        );

        // Layer 1: Just AND gates (base case - no child)
        let mut current_layer: Vec<RecAnd<Match<'a>>> = all_and_gates
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

        // Keep building layers until we can't find any more matches
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
}

fn rec_and_cells<'a, 'ctx>(rec_and: &'a RecAnd<Match<'ctx>>) -> Vec<&'a CellWrapper<'ctx>> {
    let mut cells = Vec::new();
    let and_cell = &rec_and.and.y.inner;
    cells.push(and_cell);

    if let Some(ref child) = rec_and.child {
        cells.extend(rec_and_cells(child));
    }

    cells
}

fn build_next_layer<'ctx>(
    path: &Instance,
    all_and_gates: &[AndGate<Match<'ctx>>],
    prev_layer: &[RecAnd<Match<'ctx>>],
    haystack_index: &GraphIndex<'ctx>,
) -> Vec<RecAnd<Match<'ctx>>> {
    let start_time = std::time::Instant::now();
    let mut next_layer = Vec::new();
    let mut candidates_checked = 0;
    let mut validations_passed = 0;

    for prev in prev_layer {
        let top_and_cell = &prev.and.y.inner;
        let fanout = haystack_index
            .fanout_set(top_and_cell)
            .expect("Fanout not found for cell");
        let contained_cells = rec_and_cells(prev);

        for and_gate in all_and_gates {
            let cell = &and_gate.y.inner;

            if !fanout.contains(cell) || contained_cells.contains(&cell) {
                continue;
            }

            candidates_checked += 1;

            // Update child's path to be under "child"
            let mut child = prev.clone();
            update_rec_and_path(&mut child, path.child("child"));

            let candidate = RecAnd {
                path: path.clone(),
                and: and_gate.clone(),
                child: Some(Box::new(child)),
            };

            // Validate connections manually since we constructed it manually
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

fn update_rec_and_path<'ctx>(rec_and: &mut RecAnd<Match<'ctx>>, new_path: Instance) {
    rec_and.path = new_path.clone();
    let and_path = new_path.child("and");
    rec_and.and.path = and_path.clone();
    rec_and.and.a.path = and_path.child("a");
    rec_and.and.b.path = and_path.child("b");
    rec_and.and.y.path = and_path.child("y");

    // Recursively update nested children
    if let Some(ref mut child) = rec_and.child {
        update_rec_and_path(child, new_path.child("child"));
    }
}
