
use svql_driver::{Driver, SubgraphMatch};

use crate::{Search, State, WithPath};
use crate::instance::Instance;

pub trait Netlist<S>: WithPath<S> where S: State {
    // --- Constants ---
    const MODULE_NAME        : &'static str;
    const FILE_PATH          : &'static str;

    fn driver(&self) -> &svql_driver::Driver;
}

pub trait SearchableNetlist<'p, 'd>: Netlist<Search> {
    type Hit;
    fn from_query_match(match_: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit;
    fn query(&self, haystack: &Driver, path: Instance) -> Vec<Self::Hit> {
        svql_driver::subgraph::find_subgraphs(&self.driver().design().read().unwrap(), &haystack.design().read().unwrap())
            .into_iter()
            .map(|m| Self::from_query_match(m, path.clone()))
            .collect()
    }
}