use std::collections::HashSet;

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
        composite::{Composite, MatchedComposite, SearchableComposite, filter_out_by_connection},
        netlist::SearchableNetlist,
    }, // FIXED: traits::netlist
};

#[derive(Debug, Clone, PartialEq)]
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
        let max_layers = config.max_recursion_depth;

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
            if let Some(max) = max_layers {
                if layer_num > max {
                    tracing::event!(
                        tracing::Level::INFO,
                        "RecOr::query: Reached max recursion depth of {}, stopping", // CHANGED: RecOr
                        max
                    );
                    break;
                }
            }
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

impl<'ctx> RecOr<Match<'ctx>> {
    pub fn fanin_set(&self, haystack_index: &GraphIndex<'ctx>) -> HashSet<CellWrapper<'ctx>> {
        let mut all_cells = HashSet::new();
        self.collect_cells(&mut all_cells);
        let mut fanin = HashSet::new();
        for cell in &all_cells {
            if let Some(fanin_set) = haystack_index.fanin_set(cell) {
                fanin.extend(fanin_set.iter().cloned());
            }
        }
        fanin
    }

    fn collect_cells(&self, cells: &mut HashSet<CellWrapper<'ctx>>) {
        let or_cell = self
            .or
            .y
            .val
            .as_ref()
            .expect("OR cell not found")
            .design_node_ref
            .as_ref()
            .expect("Design node not found");
        cells.insert(or_cell.clone());
        if let Some(ref child) = self.child {
            child.collect_cells(cells);
        }
    }
}

fn rec_or_cells<'a, 'ctx>(rec_or: &'a RecOr<Match<'ctx>>) -> Vec<&'a CellWrapper<'ctx>> {
    let mut cells = Vec::new();
    let or_cell: &CellWrapper<'ctx> = rec_or
        .or
        .y
        .val
        .as_ref()
        .expect("Or cell not found")
        .design_node_ref
        .as_ref()
        .expect("Design node not found");
    cells.push(or_cell);

    if let Some(ref child) = rec_or.child {
        cells.extend(rec_or_cells(child));
    }

    cells
}

fn build_next_layer<'ctx>(
    path: &Instance,
    all_or_gates: &[OrGate<Match<'ctx>>],
    prev_layer: &[RecOr<Match<'ctx>>],
    haystack_index: &GraphIndex<'ctx>,
) -> Vec<RecOr<Match<'ctx>>> {
    let start_time = std::time::Instant::now();

    tracing::event!(
        tracing::Level::INFO,
        "build_next_layer: Starting for path={}, prev_layer_size={}, all_or_gates_size={}",
        path.inst_path(),
        prev_layer.len(),
        all_or_gates.len()
    );

    // Define connections for OR gate inputs (commutative: check both 'a' and 'b')
    // Note: Connections are defined at search-time but used for filtering match-time instances
    let temp_prev = RecOr::<Search>::new(path.clone()); // Dummy for connection definition
    let temp_or_gate = OrGate::<Search>::new(path.child("temp".to_string())); // Dummy for connection definition
    let conn_to_a = Connection {
        from: temp_prev.or.y.clone(),
        to: temp_or_gate.a.clone(),
    };
    let conn_to_b = Connection {
        from: temp_prev.or.y.clone(),
        to: temp_or_gate.b.clone(),
    };

    // Get candidate pairs using filter_out_by_connection (handles parallelism and cell extraction)
    let pairs_to_a: Vec<(RecOr<Match<'ctx>>, OrGate<Match<'ctx>>)> = filter_out_by_connection(
        haystack_index,
        conn_to_a,
        prev_layer.to_vec(),
        all_or_gates.to_vec(),
    );
    let pairs_to_b: Vec<(RecOr<Match<'ctx>>, OrGate<Match<'ctx>>)> = filter_out_by_connection(
        haystack_index,
        conn_to_b,
        prev_layer.to_vec(),
        all_or_gates.to_vec(),
    );

    // Combine and deduplicate pairs (since 'a' and 'b' are commutative, avoid double-counting)
    let mut all_pairs = pairs_to_a;
    all_pairs.extend(pairs_to_b);
    all_pairs.sort_by(|(p1, o1), (p2, o2)| {
        (p1.path.inst_path().cmp(&p2.path.inst_path()))
            .then(o1.path.inst_path().cmp(&o2.path.inst_path()))
    });
    all_pairs.dedup(); // Remove duplicates based on paths

    let mut candidates_checked = 0;
    let mut validations_passed = 0;
    let validation_start = std::time::Instant::now();

    // Process pairs: apply additional filters and build candidates
    let next_layer: Vec<RecOr<Match<'ctx>>> = all_pairs
        .into_iter()
        .filter_map(|(prev, or_gate)| {
            candidates_checked += 1;

            // Exclude if the candidate OR gate's output cell is already in the previous RecOr's cells (avoid cycles)
            let contained_cells = rec_or_cells(&prev);
            let or_gate_cell = or_gate
                .y
                .val
                .as_ref()
                .expect("Or cell not found")
                .design_node_ref
                .as_ref()
                .expect("Design node not found");
            if contained_cells.contains(&or_gate_cell) {
                return None;
            }

            // Update child's path to be under "child"
            let mut child = prev.clone();
            update_rec_or_path(&mut child, path.child("child".to_string()));

            let candidate = RecOr {
                path: path.clone(),
                or: or_gate.clone(),
                child: Some(Box::new(child)),
            };

            // Validate connections (additional check beyond fanout)
            if candidate.validate_connections(candidate.connections(), haystack_index) {
                validations_passed += 1;
                Some(candidate)
            } else {
                None
            }
        })
        .collect();

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
