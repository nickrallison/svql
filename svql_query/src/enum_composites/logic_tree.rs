// svql_query/src/enum_composites/logic_tree.rs
use crate::{
    Connection, Match, Search, State, Wire, WithPath,
    enum_composites::combinational::Combinational,
    instance::Instance,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        enum_composite::SearchableEnumComposite,
    },
};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

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
    /// Recursive children - each input can be another LogicTree or a leaf (None)
    /// Length always matches root_gate.num_inputs()
    pub inputs: Vec<Option<Box<LogicTree<S>>>>,
    pub depth: usize,
}

impl<S> LogicTree<S>
where
    S: State,
{
    /// Create a leaf LogicTree (gate with all external inputs)
    pub fn new(path: Instance, root_gate: Combinational<S>) -> Self {
        let num_inputs = root_gate.num_inputs();
        Self {
            path: path.clone(),
            root_gate,
            inputs: vec![None; num_inputs],
            depth: 1,
        }
    }

    /// Create a LogicTree with specific children at each input
    pub fn with_children(
        path: Instance,
        root_gate: Combinational<S>,
        inputs: Vec<Option<Box<LogicTree<S>>>>,
    ) -> Self {
        assert_eq!(
            inputs.len(),
            root_gate.num_inputs(),
            "Number of inputs must match gate's input count"
        );

        let max_child_depth = inputs
            .iter()
            .filter_map(|opt| opt.as_ref().map(|tree| tree.depth))
            .max()
            .unwrap_or(0);

        Self {
            path,
            root_gate,
            inputs,
            depth: 1 + max_child_depth,
        }
    }

    /// Get the output wire of the root gate
    pub fn output(&self) -> &Wire<S> {
        self.root_gate.y()
    }

    /// Check if this is a leaf node (all inputs are external)
    pub fn is_leaf(&self) -> bool {
        self.inputs.iter().all(|opt| opt.is_none())
    }

    /// Count total number of gates in the tree
    pub fn gate_count(&self) -> usize {
        1 + self
            .inputs
            .iter()
            .filter_map(|opt| opt.as_ref())
            .map(|tree| tree.gate_count())
            .sum::<usize>()
    }

    /// Count number of leaf inputs (external signals)
    pub fn leaf_input_count(&self) -> usize {
        self.inputs.iter().filter(|opt| opt.is_none()).count()
            + self
                .inputs
                .iter()
                .filter_map(|opt| opt.as_ref())
                .map(|tree| tree.leaf_input_count())
                .sum::<usize>()
    }

    /// Get descriptive summary of the tree structure
    pub fn describe(&self) -> String {
        format!(
            "{} tree: depth={}, gates={}, leaves={}",
            self.root_gate.gate_type(),
            self.depth,
            self.gate_count(),
            self.leaf_input_count()
        )
    }
}

impl<S> WithPath<S> for LogicTree<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        // First check root gate
        if let Some(port) = self.root_gate.find_port(p) {
            return Some(port);
        }

        // Then recursively check children
        for child_opt in &self.inputs {
            if let Some(child) = child_opt {
                if let Some(port) = child.find_port(p) {
                    return Some(port);
                }
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
        let mut all_connections = Vec::new();

        // Connect each child's output to this gate's corresponding input
        let root_inputs = self.root_gate.get_inputs();

        for (i, child_opt) in self.inputs.iter().enumerate() {
            if let Some(child) = child_opt {
                if i < root_inputs.len() {
                    all_connections.push(vec![Connection {
                        from: child.output().clone(),
                        to: root_inputs[i].clone(),
                    }]);
                }

                // Recursively add child's internal connections
                all_connections.extend(child.connections());
            }
        }

        all_connections
    }
}

impl<'ctx> MatchedComposite<'ctx> for LogicTree<Match<'ctx>> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool + 'ctx>> {
        vec![
            // Ensure reasonable depth bounds
            Box::new(|tree: &LogicTree<Match<'ctx>>| tree.depth <= 10),
            // Ensure we have at least one gate
            Box::new(|tree: &LogicTree<Match<'ctx>>| tree.gate_count() >= 1),
        ]
    }
}

impl SearchableComposite for LogicTree<Search> {
    type Hit<'ctx> = LogicTree<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Use Combinational's context (which already merges all gate types)
        Combinational::<Search>::context(driver, config)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::info!("LogicTree::query: starting combinational logic tree search");

        // Query all combinational gates once
        let all_gates = Combinational::<Search>::query(
            haystack_key,
            context,
            path.child("root".to_string()),
            config,
        );

        tracing::info!(
            "LogicTree::query: Found {} total combinational gates",
            all_gates.len()
        );

        // Layer 1: All gates as leaves (no children)
        let mut current_layer: Vec<LogicTree<Match<'ctx>>> = all_gates
            .iter()
            .map(|gate| LogicTree::new(path.clone(), gate.clone()))
            .collect();

        tracing::info!(
            "LogicTree::query: Layer 1 (base case) has {} matches",
            current_layer.len()
        );

        let mut all_results = current_layer.clone();
        let mut layer_num = 2;
        const MAX_DEPTH: usize = 5; // Configurable depth limit

        // Iteratively build deeper trees
        loop {
            if layer_num > MAX_DEPTH {
                tracing::info!(
                    "LogicTree::query: Reached max depth {}, stopping",
                    MAX_DEPTH
                );
                break;
            }

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

/// Build next layer of trees by connecting previous layer as children to new gates
fn build_next_layer<'ctx>(
    path: &Instance,
    all_gates: &[Combinational<Match<'ctx>>],
    prev_layer: &[LogicTree<Match<'ctx>>],
) -> Vec<LogicTree<Match<'ctx>>> {
    let mut next_layer = Vec::new();

    for gate in all_gates {
        let num_inputs = gate.num_inputs();

        // Try each previous tree as a child at each input position
        for input_idx in 0..num_inputs {
            for prev_tree in prev_layer {
                // Create inputs vector with child at specific position
                let mut inputs: Vec<Option<Box<LogicTree<Match<'ctx>>>>> = vec![None; num_inputs];

                // Clone and update child path
                let mut child = prev_tree.clone();
                update_tree_path(&mut child, path.child(format!("input_{}", input_idx)));

                inputs[input_idx] = Some(Box::new(child));

                let candidate =
                    LogicTree::with_children(path.clone(), gate.clone(), inputs.clone());

                // Validate connections
                if candidate.validate_connections(candidate.connections()) {
                    next_layer.push(candidate);
                }
            }
        }
    }

    next_layer
}

/// Recursively update all paths in a tree
fn update_tree_path<'ctx>(tree: &mut LogicTree<Match<'ctx>>, new_path: Instance) {
    tree.path = new_path.clone();

    // Update root gate path
    let root_path = new_path.child("root".to_string());
    update_combinational_path(&mut tree.root_gate, root_path);

    // Recursively update children
    for (i, child_opt) in tree.inputs.iter_mut().enumerate() {
        if let Some(child) = child_opt {
            update_tree_path(child, new_path.child(format!("input_{}", i)));
        }
    }
}

/// Update path for a Combinational gate (handles all variants)
fn update_combinational_path<'ctx>(gate: &mut Combinational<Match<'ctx>>, new_path: Instance) {
    match gate {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Search; // Add this import
    use crate::primitives::and::AndGate;

    #[test]
    fn test_logic_tree_leaf() {
        let root = Instance::root("test".to_string());
        let and_gate =
            Combinational::AndGate(AndGate::<Search>::new(root.child("and".to_string())));

        let tree = LogicTree::new(root.clone(), and_gate);

        assert_eq!(tree.depth, 1);
        assert!(tree.is_leaf());
        assert_eq!(tree.gate_count(), 1);
        assert_eq!(tree.leaf_input_count(), 2); // AND has 2 inputs
    }

    #[test]
    fn test_logic_tree_with_one_child() {
        let root = Instance::root("test".to_string());

        // Create child tree
        let child_and =
            Combinational::AndGate(AndGate::<Search>::new(root.child("child_and".to_string())));
        let child_tree = LogicTree::new(root.child("child".to_string()), child_and);

        // Create parent tree with child at first input
        let parent_and =
            Combinational::AndGate(AndGate::<Search>::new(root.child("parent_and".to_string())));
        let parent_tree = LogicTree::with_children(
            root.clone(),
            parent_and,
            vec![Some(Box::new(child_tree)), None],
        );

        assert_eq!(parent_tree.depth, 2);
        assert!(!parent_tree.is_leaf());
        assert_eq!(parent_tree.gate_count(), 2);
        assert_eq!(parent_tree.leaf_input_count(), 3); // Child has 2, parent has 1 external
    }

    #[test]
    fn test_logic_tree_both_children() {
        let root = Instance::root("test".to_string());

        // Create two child trees
        let child1 =
            Combinational::AndGate(AndGate::<Search>::new(root.child("child1".to_string())));
        let tree1 = LogicTree::new(root.child("c1".to_string()), child1);

        let child2 =
            Combinational::AndGate(AndGate::<Search>::new(root.child("child2".to_string())));
        let tree2 = LogicTree::new(root.child("c2".to_string()), child2);

        // Parent with both children
        let parent_and =
            Combinational::AndGate(AndGate::<Search>::new(root.child("parent_and".to_string())));
        let parent_tree = LogicTree::with_children(
            root.clone(),
            parent_and,
            vec![Some(Box::new(tree1)), Some(Box::new(tree2))],
        );

        assert_eq!(parent_tree.depth, 2);
        assert_eq!(parent_tree.gate_count(), 3); // 1 parent + 2 children
        assert_eq!(parent_tree.leaf_input_count(), 4); // Each child has 2
    }

    #[test]
    fn test_logic_tree_describe() {
        let root = Instance::root("test".to_string());
        let and_gate =
            Combinational::AndGate(AndGate::<Search>::new(root.child("and".to_string())));
        let tree = LogicTree::new(root, and_gate);

        let desc = tree.describe();
        assert!(desc.contains("AND Gate"));
        assert!(desc.contains("depth=1"));
        assert!(desc.contains("gates=1"));
    }
}
