use std::path::PathBuf;

use svql_common::config::ffi::SvqlRuntimeConfig;
use crate::{QueryMatch, Search, State, WithPath};
use svql_driver_handler::Driver;
use crate::instance::Instance;

pub trait Netlist<S>: WithPath<S> where S: State {
    // --- Constants ---
    const MODULE_NAME        : &'static str;
    const FILE_PATH          : &'static str;
    const YOSYS              : &'static str;
    const SVQL_DRIVER_PLUGIN : &'static str;

    // --- Shared Functionality ---
    fn config() -> SvqlRuntimeConfig {
        let mut cfg = SvqlRuntimeConfig::default();

        let workspace_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        
        let pat_filename = PathBuf::from(Self::FILE_PATH);

        cfg.pat_filename = match pat_filename.is_absolute() {
            true => pat_filename.display().to_string(),
            false => workspace_path.join(pat_filename).display().to_string(),
        };

        cfg.pat_module_name = Self::MODULE_NAME.into();
        cfg.verbose         = true;
        cfg.max_fanout = 32;
        cfg
    }
}

pub trait SearchableNetlist: Netlist<Search> {
    type Hit;
    fn from_query_match(match_: QueryMatch, path: Instance) -> Self::Hit;
    fn query(driver:&Driver, path:Instance) -> Vec<Self::Hit> {
        driver.query(&Self::config())
            .expect("driver error")
            .map(|m| Self::from_query_match(m, path.clone()))
            .collect()
    }
}