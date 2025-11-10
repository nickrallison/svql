use std::collections::HashMap;

use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::{GraphIndex, cell::CellWrapper};

use crate::{
    Connection,
    Match,
    Search,
    State,
    WithPath,
    instance::Instance,
    primitives::or::OrGate,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        netlist::SearchableNetlist,
    }, // FIXED: traits::netlist
};

#[derive(Debug, Clone)]
pub struct RecOr<S>
where
    S: State,
{
    pub path: Instance,
    pub or: OrGate<S>, // CHANGED: 'or' instead of 'and'
    pub child: Option<Box<Self>>,
}

impl<S> RecOr<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            or: OrGate::new(path.child("or".to_string())), // CHANGED: "or" path
            child: None,
        }
    }

    pub fn with_child(path: Instance, or: OrGate<S>, child: Self) -> Self {
        Self {
            path,
            or, // CHANGED: 'or' field
            child: Some(Box::new(child)),
        }
    }

    /// Get the depth of this recursive structure (1 for just an OR gate, 2+ for nested)
    pub fn depth(&self) -> usize {
        1 + self.child.as_ref().map(|c| c.depth()).unwrap_or(0)
    }

    /// Get the output wire of the top-level OR gate
    pub fn output(&self) -> &crate::Wire<S> {
        &self.or.y
    }
}

impl<S> WithPath<S> for RecOr<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        // First, check if the path is for our OR gate
        if let Some(port) = self.or.find_port(p) {
            // CHANGED: self.or
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

impl<S> Composite<S> for RecOr<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        if let Some(ref child) = self.child {
            // Connection from child's output to this or's input (either a or b)
            vec![vec![
                Connection {
                    from: child.or.y.clone(), // CHANGED: child.or.y
                    to: self.or.a.clone(),
                },
                Connection {
                    from: child.or.y.clone(), // CHANGED: child.or.y
                    to: self.or.b.clone(),
                },
            ]]
        } else {
            // No connections for base case
            vec![]
        }
    }
}

impl<'ctx> MatchedComposite<'ctx> for RecOr<Match<'ctx>> {}

impl SearchableComposite for RecOr<Search> {
    type Hit<'ctx> = RecOr<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Only need context for OR gates
        OrGate::<Search>::context(driver, config) // CHANGED: OrGate
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::event!(
            tracing::Level::INFO,
            "RecOr::query: starting recursive OR gate search" // CHANGED: OR messages
        );

        let haystack_index = context.get(haystack_key).unwrap().index();

        // Query all OR gates once (reuse for all layers)
        let all_or_gates = OrGate::<Search>::query(
            // CHANGED: OrGate
            haystack_key,
            context,
            path.child("or".to_string()), // CHANGED: "or" path
            config,
        );

        tracing::event!(
            tracing::Level::INFO,
            "RecOr::query: Found {} total OR gates in design", // CHANGED: OR
            all_or_gates.len()
        );

        // Layer 1: Just OR gates (base case - no child)
        let mut current_layer: Vec<RecOr<Match<'ctx>>> = all_or_gates
            .iter()
            .map(|or_gate| RecOr {
                // CHANGED: RecOr / or_gate
                path: path.clone(),
                or: or_gate.clone(), // CHANGED: or
                child: None,
            })
            .collect();

        tracing::event!(
            tracing::Level::INFO,
            "RecOr::query: Layer 1 (base case) has {} matches", // CHANGED: RecOr
            current_layer.len()
        );

        let mut all_results = current_layer.clone();
        let mut layer_num = 2;

        // Keep building layers until we can't find any more matches
        loop {
            tracing::event!(
                tracing::Level::INFO,
                "RecOr::query: Building layer {}", // CHANGED: RecOr
                layer_num
            );
            let next_layer = build_next_layer(&path, &all_or_gates, &current_layer, haystack_index);

            if next_layer.is_empty() {
                tracing::event!(
                    tracing::Level::INFO,
                    "RecOr::query: No more matches at layer {}, stopping", // CHANGED: RecOr
                    layer_num
                );
                break;
            }

            tracing::event!(
                tracing::Level::INFO,
                "RecOr::query: Layer {} has {} matches", // CHANGED: RecOr
                layer_num,
                next_layer.len()
            );

            all_results.extend(next_layer.iter().cloned());
            current_layer = next_layer;
            layer_num += 1;
        }

        tracing::event!(
            tracing::Level::INFO,
            "RecOr::query: Total {} matches across {} layers", // CHANGED: RecOr
            all_results.len(),
            layer_num - 1
        );

        all_results
    }
}

fn build_next_layer<'ctx>(
    path: &Instance,
    all_or_gates: &[OrGate<Match<'ctx>>],
    prev_layer: &[RecOr<Match<'ctx>>],
    haystack_index: &GraphIndex<'ctx>,
) -> Vec<RecOr<Match<'ctx>>> {
    let mut next_layer = Vec::new();

    tracing::event!(
        tracing::Level::INFO,
        "build_next_layer: Building hashmap of fanouts from previous layer"
    );
    let mut rec_out_fanout_map: HashMap<CellWrapper<'ctx>, Vec<usize>> = HashMap::new();
    for index in 0..prev_layer.len() {
        let rec_or = &prev_layer[index];
        let top_or_cell: &CellWrapper<'ctx> = rec_or
            .or
            .y
            .val
            .as_ref()
            .expect("Or top cell not found")
            .design_node_ref
            .as_ref()
            .expect("Design node not found");
        let fanout = haystack_index
            .fanout_set(top_or_cell)
            .expect("Fanout Not found for cell");
        for fanout_cell in fanout.iter() {
            rec_out_fanout_map
                .entry(fanout_cell.clone())
                .or_insert(vec![])
                .push(index);
        }
    }

    tracing::event!(
        tracing::Level::INFO,
        "build_next_layer: Filtering OR gates that connect to previous layer"
    );

    let mut or_gates_clone = all_or_gates.to_vec();
    or_gates_clone.retain(|or_gate| {
        let or_cell: &CellWrapper<'ctx> = or_gate
            .y
            .val
            .as_ref()
            .expect("Or cell not found")
            .design_node_ref
            .as_ref()
            .expect("Design node not found");
        rec_out_fanout_map.contains_key(or_cell)
    });

    for or_gate in or_gates_clone {
        // CHANGED: or_gate
        for prev in prev_layer {
            // Update child's path to be under "child"
            let mut child = prev.clone();
            update_rec_or_path(&mut child, path.child("child".to_string())); // CHANGED: update_rec_or_path

            let candidate = RecOr {
                // CHANGED: RecOr
                path: path.clone(),
                or: or_gate.clone(), // CHANGED: or
                child: Some(Box::new(child)),
            };

            if candidate.validate_connections(candidate.connections(), haystack_index) {
                next_layer.push(candidate);
            }
        }
    }

    next_layer
}

fn update_rec_or_path<'ctx>(
    // CHANGED: rec_or
    rec_or: &mut RecOr<Match<'ctx>>, // CHANGED: RecOr
    new_path: Instance,
) {
    rec_or.path = new_path.clone();
    let or_path = new_path.child("or".to_string()); // CHANGED: "or" / or_path
    rec_or.or.path = or_path.clone(); // CHANGED: rec_or.or
    rec_or.or.a.path = or_path.child("a".to_string());
    rec_or.or.b.path = or_path.child("b".to_string());
    rec_or.or.y.path = or_path.child("y".to_string());

    // Recursively update nested children
    if let Some(ref mut child) = rec_or.child {
        update_rec_or_path(child, new_path.child("child".to_string())); // CHANGED: update_rec_or_path
    }
}
