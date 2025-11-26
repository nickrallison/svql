pub mod composite;
pub mod netlist;
pub mod variant;

use std::sync::Arc;
use svql_common::Config;
use svql_driver::{Context, DriverKey};

use crate::{Instance, Match, Search, State, Wire};

/// Base trait for all query components (Netlists, Composites, Variants, Wires).
pub trait Component<S: State> {
    fn path(&self) -> &Instance;
    fn type_name(&self) -> &'static str;
    fn children(&self) -> Vec<&dyn Component<S>>;

    fn find_port(&self, path: &Instance) -> Option<&Wire<S>>;
    fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>>;
}

/// Trait for components that can be instantiated in the Search state.
pub trait Searchable: Sized {
    fn instantiate(base_path: Instance) -> Self;
}

/// Legacy Query trait (planner WIP: new methods to be added later).
pub trait Query: Component<Search> {
    type Matched<'a>: Component<Match<'a>>;

    fn query<'a>(
        &self,
        driver: &::svql_driver::Driver,
        context: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>>;
}

// --- Topology & Connections ---
pub use crate::traits::composite::ConnectionBuilder;
pub use crate::traits::composite::Topology;

/// Validate if `from` drives `to` in haystack.
pub fn validate_connection<'ctx>(
    from: &Wire<Match<'ctx>>,
    to: &Wire<Match<'ctx>>,
    haystack_index: &svql_subgraph::GraphIndex<'ctx>,
) -> bool {
    let from_cell = &from.inner;
    let to_cell = &to.inner;

    haystack_index
        .fanout_set(from_cell)
        .map_or(false, |set| set.contains(to_cell))
}
