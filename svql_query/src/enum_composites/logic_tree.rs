use crate::State;
use crate::enum_composites::dff_any::DffAny;
use crate::instance::Instance;
use crate::primitives::and::AndAny; // Assuming you have AndAny from enum_composites
use crate::primitives::not::NotGate; // Or use an enum_composite for NOT if variants exist
use crate::primitives::or::OrAny; // Similarly for OR variants
use crate::traits::composite::{Composite, SearchableComposite};
use crate::{Connection, Wire};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

// ... (keep your existing FsmCore and FsmReg structs)

#[derive(Debug, Clone)]
pub struct LogicTree<S>
where
    S: State,
{
    pub path: Instance,
    pub root_gate: Gate<S>, // Root of the logic tree (e.g., final AND/OR for next-state computation)
    pub inputs: Vec<Wire<S>>, // Inputs to the tree (from FSM inputs or other regs)
    pub outputs: Vec<Wire<S>>, // Outputs from the tree (to DFF D pins)
    pub depth: usize,       // Bounded depth to prevent over-matching
}

#[derive(Debug, Clone)]
pub enum Gate<S>
where
    S: State,
{
    And(AndAny<S>),
    Or(OrAny<S>),
    Not(NotGate<S>),
    // Add more: Xor, Mux, etc., as needed
    Composite(Box<LogicTree<S>>), // For nested sub-trees
}

impl<S> LogicTree<S>
where
    S: State,
{
    pub fn new(path: Instance, max_depth: usize) -> Self {
        Self {
            path: path.clone(),
            root_gate: Gate::And(AndAny::new(path.child("root".to_string()))), // Default to AND; query will refine
            inputs: vec![],
            outputs: vec![],
            depth: max_depth,
        }
    }

    /// Add an input wire (e.g., from FSM primary input or another reg's Q)
    pub fn add_input(&mut self, wire: Wire<S>) {
        self.inputs.push(wire);
    }

    /// Add an output wire (e.g., to a DFF's D input)
    pub fn add_output(&mut self, wire: Wire<S>) {
        self.outputs.push(wire);
    }

    /// Get the primary output (e.g., next-state signal)
    pub fn primary_output(&self) -> Option<&Wire<S>> {
        self.outputs.first()
    }
}

impl<S> WithPath<S> for LogicTree<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        // Delegate to root_gate or inputs/outputs based on path
        match p.get_item(self.path.height() + 1).as_deref() {
            Some("root") => self.root_gate.find_port(p),
            Some("inputs") => self.inputs.iter().find_map(|w| w.find_port(p)),
            Some("outputs") => self.outputs.iter().find_map(|w| w.find_port(p)),
            _ => None,
        }
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
        let mut conns = vec![];

        // Connect inputs to root_gate inputs (fan-out as needed)
        for input in &self.inputs {
            conns.push(vec![Connection {
                from: input.clone(),
                to: self.root_gate.input_wire().clone(), // Assume Gate has input_wire()
            }]);
        }

        // Connect root_gate output to tree outputs
        if let Some(output) = self.primary_output() {
            conns.push(vec![Connection {
                from: self.root_gate.output_wire().clone(), // Assume Gate has output_wire()
                to: output.clone(),
            }]);
        }

        // Recursive connections if Composite variant
        if let Gate::Composite(subtree) = &self.root_gate {
            conns.extend_from_slice(&subtree.connections());
        }

        conns
    }
}

// Assume Gate needs WithPath/Composite impls (extend your primitives)
impl<S> Gate<S>
where
    S: State,
{
    fn input_wire(&self) -> Wire<S> {
        // Delegate to inner (e.g., AndAny.a or .b; simplify to first input for now)
        match self {
            Gate::And(and) => and.a.clone(), // Or handle multi-input
            Gate::Or(or) => or.a.clone(),
            Gate::Not(not) => not.a.clone(),
            Gate::Composite(tree) => tree.inputs.first().cloned().unwrap_or_default(),
        }
    }

    fn output_wire(&self) -> Wire<S> {
        match self {
            Gate::And(and) => and.y.clone(),
            Gate::Or(or) => or.y.clone(),
            Gate::Not(not) => not.y.clone(),
            Gate::Composite(tree) => tree.primary_output().cloned().unwrap_or_default(),
        }
    }
}

// For MatchedComposite (validation in Match<'ctx>)
impl<'ctx> MatchedComposite<'ctx> for LogicTree<Match<'ctx>> {
    // Add filters, e.g., ensure no sequential elements in logic tree
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool + 'ctx>> {
        vec![Box::new(|tree: &LogicTree<Match<'ctx>>| {
            // Example: Ensure depth is bounded and no DFFs in tree
            tree.depth <= 5 && !tree.contains_sequential()
        })]
    }
}

impl<'ctx> LogicTree<Match<'ctx>> {
    fn contains_sequential(&self) -> bool {
        // Traverse tree to check for DFF cells (using design_node_ref)
        false // Placeholder: implement cell type check
    }
}

// SearchableComposite impl for querying logic trees
impl SearchableComposite for LogicTree<Search> {
    type Hit<'ctx> = LogicTree<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Merge contexts for all gate types
        let and_ctx = AndAny::<Search>::context(driver, config)?;
        let or_ctx = OrAny::<Search>::context(driver, config)?; // Assume OrAny exists
        let not_ctx = NotGate::<Search>::context(driver, config)?;
        Ok(and_ctx.merge(or_ctx).merge(not_ctx))
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        // Step 1: Query root gates (e.g., final AND/OR for next-state)
        let root_gates: Vec<_> = AndAny::<Search>::query(
            // Or use a union query
            haystack_key,
            context,
            path.child("root".to_string()),
            config,
        )
        .into_iter()
        .map(|and| Gate::And(and))
        .chain(
            OrAny::<Search>::query(
                haystack_key,
                context,
                path.child("root".to_string()),
                config,
            )
            .into_iter()
            .map(|or| Gate::Or(or)),
        )
        .collect();

        // Step 2: For each root, recursively build tree (bounded depth)
        let mut results = vec![];
        for root_gate in root_gates {
            if let Some(tree) = build_tree_recursive(
                haystack_key,
                context,
                path.clone(),
                root_gate,
                1,                                   // Current depth
                config.max_logic_depth.unwrap_or(5), // Configurable bound
            ) {
                results.push(tree);
            }
        }

        results
    }
}

/// Recursive builder: Extend tree by finding child gates connected to current output
fn build_tree_recursive<'ctx>(
    haystack_key: &DriverKey,
    context: &'ctx Context,
    path: Instance,
    current_gate: Gate<Search>,
    current_depth: usize,
    max_depth: usize,
) -> Option<LogicTree<Match<'ctx>>> {
    if current_depth > max_depth {
        return None; // Bound recursion
    }

    // Query child gates connected to current_gate's output
    let child_candidates = query_connected_gates(
        // Implement this: subgraph query on fan-out
        haystack_key,
        context,
        current_gate.output_wire().path.clone(),
        config,
    );

    let mut tree = LogicTree::new(path, max_depth);
    tree.root_gate = current_gate;

    // Add children recursively (simplify: pick first valid child for tree structure)
    if let Some(child_gate) = child_candidates.first().cloned() {
        if let Some(child_tree) = build_tree_recursive(
            haystack_key,
            context,
            path.child("child".to_string()),
            child_gate,
            current_depth + 1,
            max_depth,
        ) {
            tree.root_gate = Gate::Composite(Box::new(child_tree));
        }
    }

    // TODO: Bind inputs/outputs based on fan-in/fan-out
    Some(tree)
}

/// Placeholder: Query gates connected to a specific wire (fan-out subgraph)
fn query_connected_gates<'ctx>(
    haystack_key: &DriverKey,
    context: &'ctx Context,
    from_wire_path: Instance,
    config: &Config,
) -> Vec<Gate<Match<'ctx>>> {
    // Implement: Use SVQL subgraph matcher to find gates where input connects to from_wire_path
    // Return matched gates as enum variants
    vec![] // Placeholder
}
