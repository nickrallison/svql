
use svql_driver::read_input_to_design;
use svql_driver::SubgraphMatch;

use crate::{Search, State, WithPath};
use crate::instance::Instance;

pub trait Netlist<S>: WithPath<S> where S: State {
    // --- Constants ---
    const MODULE_NAME        : &'static str;
    const FILE_PATH          : &'static str;
    const YOSYS              : &'static str;
    const SVQL_DRIVER_PLUGIN : &'static str;

    fn design() -> Result<prjunnamed_netlist::Design, Box<dyn std::error::Error>> {
        read_input_to_design(None, Self::FILE_PATH.to_string())
    }
    fn config() -> svql_driver::config::Config {
        todo!()
    }
}

pub trait SearchableNetlist: Netlist<Search> {
    type Hit;
    fn from_query_match(match_: SubgraphMatch, path: Instance) -> Self::Hit;
    fn query<'n, 'h>(haystack: &'h prjunnamed_netlist::Design, path: Instance) -> Vec<Self::Hit> {
        driver.query(&Self::config())
            .into_iter()
            .map(|m| Self::from_query_match(m, path.clone()))
            .collect()
    }
}