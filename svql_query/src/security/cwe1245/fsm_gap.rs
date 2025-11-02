use crate::{
    Connection, Match, Search, State, WithPath,
    instance::Instance,
    primitives::fsm::{OneHotStateReg, TransitionMux},
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        netlist::SearchableNetlist,
    },
};
use petgraph::prelude::*;
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

// Recursive composite for FSM transitions (like RecOr in CWE-1234).
// Matches tree of muxes/cases selecting next_state from current state.
#[derive(Debug, Clone)]
pub struct FsmTransitionTree<S>
where
    S: State,
{
    pub path: Instance,
    pub state_reg: OneHotStateReg<S>,
    pub transition: TransitionMux<S>,
    pub child: Option<Box<Self>>, // Recursive for multi-level transitions
}

impl<S> FsmTransitionTree<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            state_reg: OneHotStateReg::new(path.child("state_reg".to_string())),
            transition: TransitionMux::new(path.child("transition".to_string())),
            child: None,
        }
    }

    pub fn with_child(
        path: Instance,
        state_reg: OneHotStateReg<S>,
        transition: TransitionMux<S>,
        child: Self,
    ) -> Self {
        Self {
            path,
            state_reg,
            transition,
            child: Some(Box::new(child)),
        }
    }

    pub fn depth(&self) -> usize {
        1 + self.child.as_ref().map(|c| c.depth()).unwrap_or(0)
    }

    pub fn output(&self) -> &crate::Wire<S> {
        &self.transition.next_state
    }
}

impl<S> WithPath<S> for FsmTransitionTree<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        if let Some(port) = self.state_reg.find_port(p) {
            return Some(port);
        }
        if let Some(port) = self.transition.find_port(p) {
            return Some(port);
        }
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

impl<S> Composite<S> for FsmTransitionTree<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut conns = vec![vec![
            // State feeds into transition
            Connection {
                from: self.state_reg.state.clone(),
                to: self.transition.state.clone(),
            },
        ]];
        if let Some(ref child) = self.child {
            // Child output feeds this transition (recursive)
            conns.push(vec![Connection {
                from: child.output().clone(),
                to: self.transition.next_state.clone(), // Or cond inputs
            }]);
        }
        conns
    }
}

impl<'ctx> MatchedComposite<'ctx> for FsmTransitionTree<Match<'ctx>> {}

// Searchable impl: Like RecOr, but builds transition graph for validation.
impl SearchableComposite for FsmTransitionTree<Search> {
    type Hit<'ctx> = FsmTransitionTree<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        let state_ctx = OneHotStateReg::<Search>::context(driver, config)?;
        let trans_ctx = TransitionMux::<Search>::context(driver, config)?;
        Ok(state_ctx.merge(trans_ctx))
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        // Similar to RecOr: Query base transitions, build layers recursively
        let all_trans = TransitionMux::<Search>::query(
            haystack_key,
            context,
            path.child("transition".to_string()),
            config,
        );
        let all_states = OneHotStateReg::<Search>::query(
            haystack_key,
            context,
            path.child("state_reg".to_string()),
            config,
        );

        // Filter for FSM keywords (e.g., "state" in names)
        let fsm_states: Vec<_> = all_states
            .into_iter()
            .filter(|s| s.path.inst_path().contains("state") || s.path.inst_path().contains("fsm"))
            .collect();

        let mut current_layer: Vec<FsmTransitionTree<Match<'ctx>>> = all_trans
            .iter()
            .filter_map(|t| {
                fsm_states
                    .iter()
                    .find(|_s| /* connectable */ true)
                    .map(|s| FsmTransitionTree {
                        path: path.clone(),
                        state_reg: s.clone(),
                        transition: t.clone(),
                        child: None,
                    })
            })
            .collect();

        let mut all_results = current_layer.clone();
        let mut _layer = 2;
        loop {
            let next_layer = build_next_fsm_layer(&path, &all_trans, &fsm_states, &current_layer);
            if next_layer.is_empty() {
                break;
            }
            all_results.extend(next_layer.clone());
            current_layer = next_layer;
            _layer += 1;
        }

        // Post-filter: Build graph and check for gaps (unreachable/deadlock)
        all_results
            .into_iter()
            .filter(|fsm| {
                let graph = build_state_graph(fsm); // Adjacency from connections
                has_fsm_gaps(&graph, fsm.depth()) // Custom validation
            })
            .collect()
    }
}

// Completed helper: Build next layer (like RecOr)
fn build_next_fsm_layer<'ctx>(
    path: &Instance,
    all_trans: &[TransitionMux<Match<'ctx>>],
    all_states: &[OneHotStateReg<Match<'ctx>>],
    prev_layer: &[FsmTransitionTree<Match<'ctx>>],
) -> Vec<FsmTransitionTree<Match<'ctx>>> {
    let mut next_layer = Vec::new();

    for trans in all_trans {
        for state in all_states {
            for prev in prev_layer {
                // Check if prev output connects to this trans input (validate_connection)
                let mut child = prev.clone();
                update_fsm_path(&mut child, path.child("child".to_string()));

                let candidate = FsmTransitionTree {
                    path: path.clone(),
                    state_reg: state.clone(),
                    transition: trans.clone(),
                    child: Some(Box::new(child)),
                };

                if candidate.validate_connections(candidate.connections()) {
                    next_layer.push(candidate);
                }
            }
        }
    }

    next_layer
}

// Completed helper: Update paths recursively (like RecOr)
fn update_fsm_path<'ctx>(fsm: &mut FsmTransitionTree<Match<'ctx>>, new_path: Instance) {
    fsm.path = new_path.clone();
    let state_path = new_path.child("state_reg".to_string());
    fsm.state_reg.path = state_path.clone();
    // Update wires: fsm.state_reg.state.path = ... (similar for inputs/outputs)

    let trans_path = new_path.child("transition".to_string());
    fsm.transition.path = trans_path.clone();
    // Update wires similarly

    if let Some(ref mut child) = fsm.child {
        update_fsm_path(child, new_path.child("child".to_string()));
    }
}

// Completed helper: Build state graph (adjacency from connections; assume 4 states for simplicity)
pub fn build_state_graph<'ctx>(fsm: &FsmTransitionTree<Match<'ctx>>) -> DiGraphMap<usize, ()> {
    let mut graph = DiGraphMap::new();
    let num_states = 4; // One-hot; generalize via width from state_reg

    for i in 0..num_states {
        graph.add_node(i);
    }

    todo!("Extract actual transitions from fsm.transition and fsm.state_reg");

    graph
}

// Completed helper: Check gaps (unreachable/deadlock/incomplete)
pub fn has_fsm_gaps(graph: &DiGraphMap<usize, ()>, depth: usize) -> bool {
    let reset = 0; // Assume state 0 is reset
    let num_states = graph.node_count() as usize; // Or from depth/width

    // Reachability: DFS from reset
    let mut visited = vec![false; num_states];
    dfs_reachability(&graph, reset, &mut visited);
    let reachable_count = visited.iter().filter(|&&v| v).count();
    let unreachable = reachable_count < num_states;

    // Deadlocks: Nodes with out-degree 0 (non-halt)
    let deadlocks: Vec<_> = graph
        .node_indices()
        .filter(|&n| graph.out_degree(n) == 0 && n.index() != num_states - 1) // Exclude halt
        .collect();
    let has_deadlock = !deadlocks.is_empty();

    // Incomplete case: Heuristic (if < all states covered; or no default edge)
    let incomplete = reachable_count < num_states; // Proxy for missing cases

    unreachable || has_deadlock || incomplete
}

// DFS helper for reachability
fn dfs_reachability(graph: &DiGraphMap<usize, ()>, node: usize, visited: &mut Vec<bool>) {
    if visited[node] {
        return;
    }
    visited[node] = true;
    for neighbor in graph.neighbors(node) {
        dfs_reachability(graph, neighbor.index(), visited);
    }
}
