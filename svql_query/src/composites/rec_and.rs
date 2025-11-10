use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::{GraphIndex, cell::CellWrapper};

use crate::{
    Connection, Match, Search, State, WithPath,
    instance::Instance,
    primitives::and::AndGate,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        netlist::SearchableNetlist,
    },
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
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            and: AndGate::new(path.child("and".to_string())),
            child: None,
        }
    }

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
    pub fn output(&self) -> &crate::Wire<S> {
        &self.and.y
    }
}

impl<S> WithPath<S> for RecAnd<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
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

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for RecAnd<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        if let Some(ref child) = self.child {
            // Connection from child's output to this and's input (either a or b)
            vec![vec![
                Connection {
                    from: child.and.y.clone(),
                    to: self.and.a.clone(),
                },
                Connection {
                    from: child.and.y.clone(),
                    to: self.and.b.clone(),
                },
            ]]
        } else {
            // No connections for base case
            vec![]
        }
    }
}

impl<'ctx> MatchedComposite<'ctx> for RecAnd<Match<'ctx>> {}

impl<'ctx> RecAnd<Match<'ctx>> {
    /// Check if the NOT gate output connects to any level of the RecOr tree.
    /// This is the key validation for the CWE1234 pattern: there must be
    /// a negated lock signal somewhere in the bypass conditions.
    pub fn has_not_in_or_tree(&self, _haystack_index: &GraphIndex<'ctx>) -> bool {
        // Placeholder - RecAnd doesn't use NOT gates directly, but kept for consistency
        true
    }

    /// Get the depth of the AND tree for debugging/reporting
    pub fn and_tree_depth(&self) -> usize {
        self.depth()
    }
}

impl SearchableComposite for RecAnd<Search> {
    type Hit<'ctx> = RecAnd<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Only need context for AND gates
        AndGate::<Search>::context(driver, config)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::query: starting recursive AND gate search"
        );

        let haystack_index = context.get(haystack_key).unwrap().index();

        // Query all AND gates once (reuse for all layers)
        let all_and_gates =
            AndGate::<Search>::query(haystack_key, context, path.child("and".to_string()), config);

        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::query: Found {} total AND gates in design",
            all_and_gates.len()
        );

        // Layer 1: Just AND gates (base case - no child)
        let mut current_layer: Vec<RecAnd<Match<'ctx>>> = all_and_gates
            .iter()
            .map(|and_gate| RecAnd {
                path: path.clone(),
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
                build_next_layer(&path, &all_and_gates, &current_layer, haystack_index);

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
    let and_cell: &CellWrapper<'ctx> = rec_and
        .and
        .y
        .val
        .as_ref()
        .expect("And cell not found")
        .design_node_ref
        .as_ref()
        .expect("Design node not found");
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

    tracing::event!(
        tracing::Level::INFO,
        "build_next_layer: Starting for path={}, prev_layer_size={}, all_and_gates_size={}",
        path.inst_path(),
        prev_layer.len(),
        all_and_gates.len()
    );

    let mut next_layer = Vec::new();

    let mut candidates_checked = 0;
    let mut validations_passed = 0;
    let validation_start = std::time::Instant::now();

    for prev in prev_layer {
        let top_and_cell: &CellWrapper<'ctx> = prev
            .and
            .y
            .val
            .as_ref()
            .expect("And top cell not found")
            .design_node_ref
            .as_ref()
            .expect("Design node not found");
        let fanout = haystack_index
            .fanout_set(top_and_cell)
            .expect("Fanout not found for cell");
        let contained_cells = rec_and_cells(prev);

        for and_gate in all_and_gates {
            let cell = &and_gate
                .y
                .val
                .as_ref()
                .expect("And cell not found")
                .design_node_ref
                .as_ref()
                .expect("Design node not found");

            if !fanout.contains(cell) || contained_cells.contains(cell) {
                continue;
            }

            candidates_checked += 1;

            // Update child's path to be under "child"
            let mut child = prev.clone();
            update_rec_and_path(&mut child, path.child("child".to_string()));

            let candidate = RecAnd {
                path: path.clone(),
                and: and_gate.clone(),
                child: Some(Box::new(child)),
            };

            if candidate.validate_connections(candidate.connections(), haystack_index) {
                validations_passed += 1;
                next_layer.push(candidate);
            }

            // Log progress every 1000 candidates to avoid spam
            if candidates_checked % 1000 == 0 {
                tracing::event!(
                    tracing::Level::DEBUG,
                    "build_next_layer: Checked {} candidates so far, {} passed validation",
                    candidates_checked,
                    validations_passed
                );
            }
        }
    }

    let validation_duration = validation_start.elapsed();
    tracing::event!(
        tracing::Level::INFO,
        "build_next_layer: Validation phase took {:?}, checked {} candidates, {} passed",
        validation_duration,
        candidates_checked,
        validations_passed
    );

    let total_duration = start_time.elapsed();
    tracing::event!(
        tracing::Level::INFO,
        "build_next_layer: Completed in {:?}, returning {} next layer items",
        total_duration,
        next_layer.len()
    );

    next_layer
}

fn update_rec_and_path<'ctx>(rec_and: &mut RecAnd<Match<'ctx>>, new_path: Instance) {
    rec_and.path = new_path.clone();
    let and_path = new_path.child("and".to_string());
    rec_and.and.path = and_path.clone();
    rec_and.and.a.path = and_path.child("a".to_string());
    rec_and.and.b.path = and_path.child("b".to_string());
    rec_and.and.y.path = and_path.child("y".to_string());

    // Recursively update nested children
    if let Some(ref mut child) = rec_and.child {
        update_rec_and_path(child, new_path.child("child".to_string()));
    }
}
