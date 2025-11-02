// svql_query/src/enum_composites/logic_tree.rs
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

use crate::{
    Connection, Match, Search, State, WithPath,
    enum_composites::combinational::Combinational,
    instance::Instance,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        enum_composite::SearchableEnumComposite,
    },
};

/// Represents a combinational logic tree with arbitrary gate types at each node
///
/// Each input to the root gate can be:
/// - None: A leaf input (external signal)
/// - Some(Box<LogicTree>): A recursive sub-tree
///
/// This enables matching complex combinational structures like:
/// - AND trees with mixed OR/XOR sub-trees
/// - Complex arithmetic/logic expressions
/// - State machine next-state logic
#[derive(Debug, Clone)]
pub struct LogicTree<S>
where
    S: State,
{
    pub path: Instance,
    pub root_gate: Combinational<S>,
    pub input_a: Option<Box<LogicTree<S>>>,
    pub input_b: Option<Box<LogicTree<S>>>,
    pub input_c: Option<Box<LogicTree<S>>>,
    pub depth: usize,
}

impl<S> LogicTree<S>
where
    S: State,
{
    /// Create a new leaf tree (just a gate, no children)
    pub fn new_leaf(path: Instance, gate: Combinational<S>) -> Self {
        Self {
            path,
            root_gate: gate,
            input_a: None,
            input_b: None,
            input_c: None,
            depth: 1,
        }
    }

    /// Get a human-readable description of the tree structure
    pub fn describe(&self) -> String {
        let gate_type = self.root_gate.gate_type();
        let child_count = self.num_children();

        if child_count == 0 {
            format!("{} (leaf)", gate_type)
        } else {
            let child_descs: Vec<_> = [&self.input_a, &self.input_b, &self.input_c]
                .iter()
                .filter_map(|opt| opt.as_ref().map(|child| child.describe()))
                .collect();
            format!("{}({})", gate_type, child_descs.join(", "))
        }
    }

    /// Get the number of children (non-None inputs)
    pub fn num_children(&self) -> usize {
        [&self.input_a, &self.input_b, &self.input_c]
            .iter()
            .filter(|opt| opt.is_some())
            .count()
    }

    /// Get the output wire of the root gate
    pub fn output(&self) -> &crate::Wire<S> {
        self.root_gate.y()
    }
}

impl<S> WithPath<S> for LogicTree<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        // First check root gate
        if let Some(port) = self.root_gate.find_port(p) {
            return Some(port);
        }

        // Then check children
        if let Some(ref child) = self.input_a {
            if let Some(port) = child.find_port(p) {
                return Some(port);
            }
        }
        if let Some(ref child) = self.input_b {
            if let Some(port) = child.find_port(p) {
                return Some(port);
            }
        }
        if let Some(ref child) = self.input_c {
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

impl<S> Composite<S> for LogicTree<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut conn_groups = Vec::new();
        let root_inputs = self.root_gate.get_inputs();

        // FIXED: Each child must connect to SOME input (alternatives within group)
        // This handles commutative gates where inputs can be in any order

        if let Some(ref child) = self.input_a {
            let mut alternatives = Vec::new();
            for input in &root_inputs {
                alternatives.push(Connection {
                    from: child.output().clone(),
                    to: input.clone(),
                });
            }
            if !alternatives.is_empty() {
                conn_groups.push(alternatives);
            }
        }

        if let Some(ref child) = self.input_b {
            let mut alternatives = Vec::new();
            for input in &root_inputs {
                alternatives.push(Connection {
                    from: child.output().clone(),
                    to: input.clone(),
                });
            }
            if !alternatives.is_empty() {
                conn_groups.push(alternatives);
            }
        }

        if let Some(ref child) = self.input_c {
            let mut alternatives = Vec::new();
            for input in &root_inputs {
                alternatives.push(Connection {
                    from: child.output().clone(),
                    to: input.clone(),
                });
            }
            if !alternatives.is_empty() {
                conn_groups.push(alternatives);
            }
        }

        conn_groups
    }
}

impl<'ctx> MatchedComposite<'ctx> for LogicTree<Match<'ctx>> {}

impl SearchableComposite for LogicTree<Search> {
    type Hit<'ctx> = LogicTree<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Need context for all combinational gates
        Combinational::<Search>::context(driver, config)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::info!("LogicTree::query: starting logic tree search");

        // Query all combinational gates once (reuse for all layers)
        let all_gates = Combinational::<Search>::query(
            haystack_key,
            context,
            path.child("root_gate".to_string()),
            config,
        );

        tracing::info!(
            "LogicTree::query: Found {} total combinational gates in design",
            all_gates.len()
        );

        // Layer 1: Just gates (base case - no children)
        let mut current_layer: Vec<LogicTree<Match<'ctx>>> = all_gates
            .iter()
            .map(|gate| LogicTree {
                path: path.clone(),
                root_gate: gate.clone(),
                input_a: None,
                input_b: None,
                input_c: None,
                depth: 1,
            })
            .collect();

        tracing::info!(
            "LogicTree::query: Layer 1 (leaves) has {} matches",
            current_layer.len()
        );

        let mut all_results = current_layer.clone();
        let mut layer_num = 2;

        // Keep building layers until we can't find any more matches
        loop {
            let next_layer = build_next_layer(&path, &all_gates, &current_layer);

            if next_layer.is_empty() {
                tracing::info!(
                    "LogicTree::query: No more matches at layer {}, stopping",
                    layer_num
                );
                break;
            }

            tracing::info!(
                "LogicTree::query: Layer {} has {} matches",
                layer_num,
                next_layer.len()
            );

            all_results.extend(next_layer.iter().cloned());
            current_layer = next_layer;
            layer_num += 1;
        }

        tracing::info!(
            "LogicTree::query: Total {} matches across {} layers",
            all_results.len(),
            layer_num - 1
        );

        all_results
    }
}

/// Build the next layer of trees by combining gates with previous layer trees
fn build_next_layer<'ctx>(
    path: &Instance,
    all_gates: &[Combinational<Match<'ctx>>],
    prev_layer: &[LogicTree<Match<'ctx>>],
) -> Vec<LogicTree<Match<'ctx>>> {
    let mut next_layer = Vec::new();

    for root_gate in all_gates {
        let num_inputs = root_gate.num_inputs();

        // We need at least one child to advance to the next layer
        match num_inputs {
            1 => {
                // Single-input gates (NOT, BUF): exactly one child
                for child_a in prev_layer {
                    let candidate = create_tree_with_children(
                        path,
                        root_gate.clone(),
                        Some(child_a.clone()),
                        None,
                        None,
                    );

                    if candidate.validate_connections(candidate.connections()) {
                        next_layer.push(candidate);
                    }
                }
            }
            2 => {
                // Dual-input gates (AND, OR, XOR, XNOR): try all combinations
                // (child, leaf)
                for child_a in prev_layer {
                    let candidate = create_tree_with_children(
                        path,
                        root_gate.clone(),
                        Some(child_a.clone()),
                        None,
                        None,
                    );

                    if candidate.validate_connections(candidate.connections()) {
                        next_layer.push(candidate);
                    }
                }

                // (leaf, child)
                for child_b in prev_layer {
                    let candidate = create_tree_with_children(
                        path,
                        root_gate.clone(),
                        None,
                        Some(child_b.clone()),
                        None,
                    );

                    if candidate.validate_connections(candidate.connections()) {
                        next_layer.push(candidate);
                    }
                }

                // (child, child)
                for child_a in prev_layer {
                    for child_b in prev_layer {
                        let candidate = create_tree_with_children(
                            path,
                            root_gate.clone(),
                            Some(child_a.clone()),
                            Some(child_b.clone()),
                            None,
                        );

                        if candidate.validate_connections(candidate.connections()) {
                            next_layer.push(candidate);
                        }
                    }
                }
            }
            3 => {
                // Triple-input gates (MUX2): all combinations with at least one child
                build_three_input_combinations(path, root_gate, prev_layer, &mut next_layer);
            }
            _ => {
                tracing::warn!("Gate with {} inputs not supported", num_inputs);
            }
        }
    }

    next_layer
}

/// Helper for building all combinations of 3-input gates
fn build_three_input_combinations<'ctx>(
    path: &Instance,
    root_gate: &Combinational<Match<'ctx>>,
    prev_layer: &[LogicTree<Match<'ctx>>],
    next_layer: &mut Vec<LogicTree<Match<'ctx>>>,
) {
    // Single child on each input
    for child_a in prev_layer {
        let candidate =
            create_tree_with_children(path, root_gate.clone(), Some(child_a.clone()), None, None);
        if candidate.validate_connections(candidate.connections()) {
            next_layer.push(candidate);
        }
    }

    for child_b in prev_layer {
        let candidate =
            create_tree_with_children(path, root_gate.clone(), None, Some(child_b.clone()), None);
        if candidate.validate_connections(candidate.connections()) {
            next_layer.push(candidate);
        }
    }

    for child_c in prev_layer {
        let candidate =
            create_tree_with_children(path, root_gate.clone(), None, None, Some(child_c.clone()));
        if candidate.validate_connections(candidate.connections()) {
            next_layer.push(candidate);
        }
    }

    // Pairs
    for child_a in prev_layer {
        for child_b in prev_layer {
            let candidate = create_tree_with_children(
                path,
                root_gate.clone(),
                Some(child_a.clone()),
                Some(child_b.clone()),
                None,
            );
            if candidate.validate_connections(candidate.connections()) {
                next_layer.push(candidate);
            }
        }
    }

    for child_a in prev_layer {
        for child_c in prev_layer {
            let candidate = create_tree_with_children(
                path,
                root_gate.clone(),
                Some(child_a.clone()),
                None,
                Some(child_c.clone()),
            );
            if candidate.validate_connections(candidate.connections()) {
                next_layer.push(candidate);
            }
        }
    }

    for child_b in prev_layer {
        for child_c in prev_layer {
            let candidate = create_tree_with_children(
                path,
                root_gate.clone(),
                None,
                Some(child_b.clone()),
                Some(child_c.clone()),
            );
            if candidate.validate_connections(candidate.connections()) {
                next_layer.push(candidate);
            }
        }
    }

    // All three
    for child_a in prev_layer {
        for child_b in prev_layer {
            for child_c in prev_layer {
                let candidate = create_tree_with_children(
                    path,
                    root_gate.clone(),
                    Some(child_a.clone()),
                    Some(child_b.clone()),
                    Some(child_c.clone()),
                );
                if candidate.validate_connections(candidate.connections()) {
                    next_layer.push(candidate);
                }
            }
        }
    }
}

/// Helper to create a tree with children, updating paths correctly
fn create_tree_with_children<'ctx>(
    path: &Instance,
    root_gate: Combinational<Match<'ctx>>,
    child_a: Option<LogicTree<Match<'ctx>>>,
    child_b: Option<LogicTree<Match<'ctx>>>,
    child_c: Option<LogicTree<Match<'ctx>>>,
) -> LogicTree<Match<'ctx>> {
    let max_child_depth = [&child_a, &child_b, &child_c]
        .iter()
        .filter_map(|opt| opt.as_ref().map(|child| child.depth))
        .max()
        .unwrap_or(0);

    let child_a = child_a.map(|mut c| {
        update_tree_path(&mut c, path.child("input_a".to_string()));
        Box::new(c)
    });

    let child_b = child_b.map(|mut c| {
        update_tree_path(&mut c, path.child("input_b".to_string()));
        Box::new(c)
    });

    let child_c = child_c.map(|mut c| {
        update_tree_path(&mut c, path.child("input_c".to_string()));
        Box::new(c)
    });

    LogicTree {
        path: path.clone(),
        root_gate,
        input_a: child_a,
        input_b: child_b,
        input_c: child_c,
        depth: 1 + max_child_depth,
    }
}

/// Recursively update all paths in a tree
fn update_tree_path<'ctx>(tree: &mut LogicTree<Match<'ctx>>, new_path: Instance) {
    tree.path = new_path.clone();

    // Update root gate path
    let gate_path = new_path.child("root_gate".to_string());
    update_combinational_path(&mut tree.root_gate, gate_path);

    // Recursively update children
    if let Some(ref mut child) = tree.input_a {
        update_tree_path(child, new_path.child("input_a".to_string()));
    }
    if let Some(ref mut child) = tree.input_b {
        update_tree_path(child, new_path.child("input_b".to_string()));
    }
    if let Some(ref mut child) = tree.input_c {
        update_tree_path(child, new_path.child("input_c".to_string()));
    }
}

/// Update paths in a Combinational enum variant
fn update_combinational_path<'ctx>(comb: &mut Combinational<Match<'ctx>>, new_path: Instance) {
    match comb {
        Combinational::AndGate(g) => {
            g.path = new_path.clone();
            g.a.path = new_path.child("a".to_string());
            g.b.path = new_path.child("b".to_string());
            g.y.path = new_path.child("y".to_string());
        }
        Combinational::Or(g) => {
            g.path = new_path.clone();
            g.a.path = new_path.child("a".to_string());
            g.b.path = new_path.child("b".to_string());
            g.y.path = new_path.child("y".to_string());
        }
        Combinational::Xor(g) => {
            g.path = new_path.clone();
            g.a.path = new_path.child("a".to_string());
            g.b.path = new_path.child("b".to_string());
            g.y.path = new_path.child("y".to_string());
        }
        Combinational::Xnor(g) => {
            g.path = new_path.clone();
            g.a.path = new_path.child("a".to_string());
            g.b.path = new_path.child("b".to_string());
            g.y.path = new_path.child("y".to_string());
        }
        Combinational::Not(g) => {
            g.path = new_path.clone();
            g.a.path = new_path.child("a".to_string());
            g.y.path = new_path.child("y".to_string());
        }
        Combinational::Buf(g) => {
            g.path = new_path.clone();
            g.a.path = new_path.child("a".to_string());
            g.y.path = new_path.child("y".to_string());
        }
        Combinational::Mux2(g) => {
            g.path = new_path.clone();
            g.a.path = new_path.child("a".to_string());
            g.b.path = new_path.child("b".to_string());
            g.sel.path = new_path.child("sel".to_string());
            g.y.path = new_path.child("y".to_string());
        }
    }
}
