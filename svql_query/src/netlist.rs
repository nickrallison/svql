
use std::sync::{Arc, RwLock, RwLockReadGuard};

use prjunnamed_netlist::Design;
use svql_driver::{Driver, SubgraphMatch};

use crate::{QueryResults, Search, State, WithPath};
use crate::instance::Instance;

pub trait Netlist<S>: WithPath<S> where S: State {
    // --- Constants ---
    const MODULE_NAME        : &'static str;
    const FILE_PATH          : &'static str;

    fn driver(&self) -> &svql_driver::Driver;
}

pub trait SearchableNetlist: Netlist<Search> {
    type Hit<'p, 'd>;
    fn from_query_match<'p, 'd>(match_: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit<'p, 'd>;
    fn query<'p, 'd>(needle: &'p Driver, haystack: &'d Driver, path: Instance) -> Vec<Self::Hit<'p, 'd>> {
        svql_driver::subgraph::find_subgraphs(needle.design_as_ref(), haystack.design_as_ref())
            .into_iter()
            .map(|m| Self::from_query_match(&m, path.clone()))
            .collect()
    }
}