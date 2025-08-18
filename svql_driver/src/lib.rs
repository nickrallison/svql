
use std::{error::Error, fs::File, path::{Path, PathBuf}, sync::Arc};

use prjunnamed_netlist::Target;

pub mod driver;
pub mod subgraph;
pub mod config;
pub mod cache;
pub mod connectivity;
pub mod util;

pub use driver::Driver;
pub use subgraph::SubgraphMatch;

pub fn read_input_to_design(target: Option<Arc<dyn Target>>, name: String) -> Result<prjunnamed_netlist::Design, Box<dyn Error>> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let path = Path::new(&name);
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    };

    if name.ends_with(".uir") {
        let design = prjunnamed_netlist::parse(target, &std::fs::read_to_string(abs_path)?)?;
        Ok(design)
    } else if name.ends_with(".json") {
        let designs = prjunnamed_yosys_json::import(target, &mut File::open(abs_path)?)?;
        assert_eq!(designs.len(), 1, "can only convert single-module Yosys JSON to Unnamed IR");
        Ok(designs.into_values().next().unwrap())
    } else if name.is_empty() {
        return Err("No input file provided".into());
    } else {
        return Err(format!("Don't know what to do with input {name:?}").into());
    }
}

pub fn get_name(name: &str) -> String {
    let path = PathBuf::from(name);
    let file_stem = PathBuf::from(path.file_stem().unwrap());
    let file_name = file_stem.file_name().unwrap().to_string_lossy();
    file_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otbn_run_time() {

        let time_start = std::time::Instant::now();

        let haystack_path = "examples/larger_designs/otbn_core.json";
        let haystack_design = read_input_to_design(None, haystack_path.to_string()).expect("Failed to read input design");

        let needle_path_1 = "examples/patterns/security/access_control/locked_reg/json/async_en.json";
        let needle_design_1 = read_input_to_design(None, needle_path_1.to_string()).expect("Failed to read input design");
        let needle_name_1 = get_name(&needle_path_1);

        let needle_path_2 = "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
        let needle_design_2 = read_input_to_design(None, needle_path_2.to_string()).expect("Failed to read input design");
        let needle_name_2 = get_name(&needle_path_2);

        let needle_path_3 = "examples/patterns/security/access_control/locked_reg/json/sync_en.json";
        let needle_design_3 = read_input_to_design(None, needle_path_3.to_string()).expect("Failed to read input design");
        let needle_name_3 = get_name(&needle_path_3);

        let needle_path_4 = "examples/patterns/security/access_control/locked_reg/json/sync_mux.json";
        let needle_design_4 = read_input_to_design(None, needle_path_4.to_string()).expect("Failed to read input design");
        let needle_name_4 = get_name(&needle_path_4);

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