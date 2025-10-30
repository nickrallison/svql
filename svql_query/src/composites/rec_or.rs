use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

use crate::traits::netlist::SearchableNetlist;
use crate::{
    composites::{Composite, MatchedComposite, SearchableComposite},
    instance::Instance,
    queries::netlist::basic::or::OrGate,
    Connection,
    Match, // CHANGED: Use OrGate instead of AndGate
    Search,
    State,
    WithPath,
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
            let next_layer = build_next_layer(&path, &all_or_gates, &current_layer, layer_num); // CHANGED: all_or_gates

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
    all_or_gates: &[OrGate<Match<'ctx>>], // CHANGED: OrGate / all_or_gates
    prev_layer: &[RecOr<Match<'ctx>>],    // CHANGED: RecOr
    layer_num: usize,
) -> Vec<RecOr<Match<'ctx>>> {
    // CHANGED: RecOr
    let mut next_layer = Vec::new();

    for or_gate in all_or_gates {
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

            if candidate.validate_connections(candidate.connections()) {
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
