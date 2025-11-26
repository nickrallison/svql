pub mod composite;
pub mod netlist;
pub mod variant;

use std::sync::Arc;
use svql_common::Config;
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::GraphIndex;

use crate::{Instance, Match, State, Wire};

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

/// Trait for executing a query against the design.
pub trait Query {
    type Matched<'a>;

    fn query<'a>(
        &self,
        driver: &Driver,
        ctx: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>>;
}

// --- Topology & Connections ---

/// Implemented by Composites to define internal connectivity.
pub trait Topology<S: State> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>);
}

pub struct ConnectionBuilder<'a, S: State> {
    pub constraints: Vec<Vec<(Option<&'a Wire<S>>, Option<&'a Wire<S>>)>>,
}

impl<'a, S: State> ConnectionBuilder<'a, S> {
    pub fn connect<A, B>(&mut self, from: A, to: B)
    where
        A: Into<Option<&'a Wire<S>>>,
        B: Into<Option<&'a Wire<S>>>,
    {
        self.constraints.push(vec![(from.into(), to.into())]);
    }

    pub fn connect_any<A, B>(&mut self, options: &[(A, B)])
    where
        A: Into<Option<&'a Wire<S>>> + Clone,
        B: Into<Option<&'a Wire<S>>> + Clone,
    {
        let group = options
            .iter()
            .map(|(a, b)| (a.clone().into(), b.clone().into()))
            .collect();

        self.constraints.push(group);
    }
}

pub fn validate_connection<'ctx>(
    from: &Wire<Match<'ctx>>,
    to: &Wire<Match<'ctx>>,
    haystack_index: &GraphIndex<'ctx>,
) -> bool {
    // 'from' is the Source cell (e.g. AND gate driving Y)
    // 'to' is the Sink cell (e.g. DFF receiving D)
    // Check if 'from' drives 'to' in the haystack.
    let from_cell = &from.inner;
    let to_cell = &to.inner;

    haystack_index
        .fanout_set(from_cell)
        .map_or(false, |set| set.contains(to_cell))
}
