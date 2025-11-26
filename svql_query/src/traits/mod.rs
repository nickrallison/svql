pub mod composite;
pub mod netlist;
pub mod variant;

use std::sync::Arc;
use svql_common::Config;
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::GraphIndex;

use crate::{
    Instance, Match, Search, State, Wire,
    ir::{
        Executor, LogicalPlan, QueryDag, ResultCursor, Schema, canonicalize_to_dag,
        compute_schema_mapping,
    },
};

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

pub trait Query: Component<Search> {
    type Matched<'a>: Component<Match<'a>>;

    fn to_ir(&self, config: &Config) -> LogicalPlan; // Tree
    fn dag_ir(&self, config: &Config) -> QueryDag {
        canonicalize_to_dag(self.to_ir(config))
    } // DAG

    fn reconstruct<'a>(&self, cursor: &mut ResultCursor<'a>) -> Self::Matched<'a>;
    fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize>;
    fn expected_schema(&self) -> Schema;

    // Default: Optimized query via DAG
    fn query<'a, T: Executor>(
        &self,
        executor: &'a T,
        ctx: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>> {
        let dag = self.dag_ir(config);
        let exec_res = executor.execute_dag(&dag, ctx);
        let expected = self.expected_schema();
        let mapping = compute_schema_mapping(&expected, &exec_res.schema);
        exec_res
            .rows
            .map(move |row| {
                let mut cursor = ResultCursor::new(&row, &mapping);
                self.reconstruct(&mut cursor)
            })
            .collect()
    }
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
