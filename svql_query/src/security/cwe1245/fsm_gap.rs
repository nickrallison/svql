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

// Helper: Build next layer (similar to RecOr's build_next_layer)
pub fn build_next_fsm_layer<'ctx>(
    _path: &Instance,
    _all_trans: &[TransitionMux<Match<'ctx>>],
    _fsm_states: &[OneHotStateReg<Match<'ctx>>],
    _current_layer: &[FsmTransitionTree<Match<'ctx>>],
) -> Vec<FsmTransitionTree<Match<'ctx>>> {
    // TODO: Implement similar to RecOr
    vec![]
}

// Helper: Build state graph from connections (simplified adj list)
pub fn build_state_graph<'ctx>(_fsm: &FsmTransitionTree<Match<'ctx>>) -> DiGraphMap<usize, ()> {
    let graph = DiGraphMap::new();
    // Extract states from state_reg, edges from transitions/connections
    // E.g., graph.add_edge(current_state_id, next_state_id, ());
    graph
}

// Helper: Check for gaps (unreachable from reset, deadlocks, incomplete cases)
pub fn has_fsm_gaps(_graph: &DiGraphMap<usize, ()>, _depth: usize) -> bool {
    // TODO: Implement reachability check
    false
}
