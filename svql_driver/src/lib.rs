
use std::{error::Error, fs::File, path::{Path, PathBuf}, sync::Arc};

use prjunnamed_netlist::Target;

pub mod driver;
pub mod subgraph;
pub mod config;
pub mod cache;
pub mod util;

pub use driver::Driver;
pub use subgraph::SubgraphMatch;


#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::{cache::Cache, util::{load_driver_cached, load_driver_from}};

    use super::*;

    #[test]
    fn test_otbn_run_time() {

        let cache = Arc::new(Mutex::new(Cache::new()));


        let haystack_path = "examples/larger_designs/otbn_core.json";
        let haystack_design = load_driver_cached("examples/larger_designs/otbn_core.json", cache.clone())
            .expect("Failed to read input design");

        let (and_design, _) = load_driver_cached("examples/patterns/basic/and/and_gate.v", cache.clone()).expect("Failed to read input design");

        let needle_path_1 = "examples/patterns/security/access_control/locked_reg/json/async_en.json";
        let needle_design_1 = load_driver_cached(needle_path_1, cache.clone()).expect("Failed to read input design");

        let needle_path_2 = "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
        let needle_design_2 = load_driver_cached(needle_path_2, cache.clone()).expect("Failed to read input design");

        let needle_path_3 = "examples/patterns/security/access_control/locked_reg/json/sync_en.json";
        let needle_design_3 = load_driver_cached(needle_path_3, cache.clone()).expect("Failed to read input design");

        let needle_path_4 = "examples/patterns/security/access_control/locked_reg/json/sync_mux.json";
        let needle_design_4 = load_driver_cached(needle_path_4, cache.clone()).expect("Failed to read input design");

        let time_start = std::time::Instant::now();

        let results_1 = subgraph::find_subgraphs(&needle_design_1, &haystack_design);
        let results_2 = subgraph::find_subgraphs(&needle_design_2, &haystack_design);
        let results_3 = subgraph::find_subgraphs(&needle_design_3, &haystack_design);
        let results_4 = subgraph::find_subgraphs(&needle_design_4, &haystack_design);

        let results_and = subgraph::find_subgraphs(&and_design, &haystack_design);

        println!("Found {} matches for needle 1", results_1.matches.len());
        println!("Found {} matches for needle 2", results_2.matches.len());
        println!("Found {} matches for needle 3", results_3.matches.len());
        println!("Found {} matches for needle 4", results_4.matches.len());
        println!("Found {} matches for AND gate", results_and.matches.len());

        let time_elapsed = time_start.elapsed();
        println!("Test completed in {:?}", time_elapsed);

        // test should take less than 2 seconds (0.2 seconds on my machine (AMD 7950X3D) in release and 1.4 seconds in debug)
        // 2000 milis in release or 14000 milis in debug
        let time_expected = if cfg!(debug_assertions) {
            std::time::Duration::from_millis(14000)
        } else {
            std::time::Duration::from_millis(2000)
        };
        assert!(time_elapsed < time_expected, "Test took too long to run, expected < {:?}, got {:?}", time_expected, time_elapsed);
    }
}