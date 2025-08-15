use std::{cell, error::Error, fs::File, path::{Path, PathBuf}, sync::Arc};

use prjunnamed_netlist::Target;
use svql_query::{composite::SearchableComposite, netlist::SearchableNetlist, queries::security::access_control::locked_reg::{sync_en::SyncEnLockedReg, sync_mux::SyncMuxLockedReg}};
use svql_driver_handler::{proc::ProcDriver, Driver};
use svql_query::{Match, Search, WithPath};

use crate::and::AndAB;

mod and;
mod subgraph; // added

fn read_input(target: Option<Arc<dyn Target>>, name: String) -> Result<prjunnamed_netlist::Design, Box<dyn Error>> {

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let path = Path::new(&name);
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    };

    if name.ends_with(".uir") {
        Ok(prjunnamed_netlist::parse(target, &std::fs::read_to_string(abs_path)?)?)
    } else if name.ends_with(".json") {
        let designs = prjunnamed_yosys_json::import(target, &mut File::open(abs_path)?)?;
        assert_eq!(designs.len(), 1, "can only convert single-module Yosys JSON to Unnamed IR");
        Ok(designs.into_values().next().unwrap())
    } else if name.is_empty() {
        panic!("no input provided")
    } else {
        panic!("don't know what to do with input {name:?}")
    }
}

fn get_name(name: &str) -> String {
    let path = PathBuf::from(name);
    let file_stem = PathBuf::from(path.file_stem().unwrap());
    let file_name = file_stem.file_name().unwrap().to_string_lossy();
    file_name.to_string()
}

fn main() {
    // env logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    // let otbn_path = PathBuf::from("examples/larger_designs/otbn.json");
    // let otbn_design = read_input(None, otbn_path.to_string_lossy().to_string()).expect("Failed to read input design");

    let haystack_path = "examples/larger_designs/otbn.json";
    // let haystack_path = "examples/patterns/security/access_control/locked_reg/json/many_locked_regs.json";
    let haystack_design = read_input(None, haystack_path.to_string()).expect("Failed to read input design");
    let haystack_name = get_name(&haystack_path);

    let needle_path = "examples/patterns/security/access_control/locked_reg/json/async_en.json";
    let needle_design = read_input(None, needle_path.to_string()).expect("Failed to read input design");
    let needle_name = get_name(&needle_path);

    // Find subgraphs using the chosen anchor kind
    let matches = subgraph::find_subgraphs(&needle_design, &haystack_design);
    println!("Found {} subgraph matches for needle {} in haystack {}.", matches.len(), needle_name, haystack_name);
}
