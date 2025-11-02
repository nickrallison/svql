pub mod fsm_gap;

use crate::security::cwe1245::fsm_gap::{FsmTransitionTree, build_state_graph, has_fsm_gaps};
use crate::traits::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::{Connection, Match, Search, State, WithPath, instance::Instance};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

// Top-level: FSM with gaps (unreachable/deadlock/incomplete).
#[derive(Debug, Clone)]
pub struct Cwe1245<S>
where
    S: State,
{
    pub path: Instance,
    pub fsm_tree: FsmTransitionTree<S>,
}

impl<S> Cwe1245<S>
where
    S: State,
{
    pub fn new(path: Instance, fsm_tree: FsmTransitionTree<S>) -> Self {
        Self {
            path: path.clone(),
            fsm_tree,
        }
    }
}

impl<S> WithPath<S> for Cwe1245<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        self.fsm_tree.find_port(p)
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for Cwe1245<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        self.fsm_tree.connections() // Delegate to sub-tree
    }
}

impl<'ctx> MatchedComposite<'ctx> for Cwe1245<Match<'ctx>> {}

impl SearchableComposite for Cwe1245<Search> {
    type Hit<'ctx> = Cwe1245<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        FsmTransitionTree::<Search>::context(driver, config)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        let trees = FsmTransitionTree::<Search>::query(haystack_key, context, path.clone(), config);
        trees
            .into_iter()
            .filter_map(|tree| {
                let candidate = Cwe1245::new(path.clone(), tree.clone());
                if candidate.has_gaps() {
                    // Custom validation
                    Some(candidate)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<'ctx> Cwe1245<Match<'ctx>> {
    pub fn has_gaps(&self) -> bool {
        let graph = build_state_graph(&self.fsm_tree); // From sub-pattern
        has_fsm_gaps(&graph, self.fsm_tree.depth())
    }
}
