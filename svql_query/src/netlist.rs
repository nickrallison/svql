
use std::sync::Arc;

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

    // --- Shared Functionality ---
    // fn config() -> Config {
    //     let mut cfg = Config::default();

    //     let workspace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        
    //     let pat_filename = PathBuf::from(Self::FILE_PATH);

    //     cfg.pat_filename = match pat_filename.is_absolute() {
    //         true => pat_filename.display().to_string(),
    //         false => workspace_path.join(pat_filename).display().to_string(),
    //     };

    //     cfg.pat_module_name = Self::MODULE_NAME.into();
    //     cfg.verbose         = true;
    //     cfg.max_fanout = 32;
    //     cfg
    // }
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