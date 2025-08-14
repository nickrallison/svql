
use std::{cell, error::Error, fs::File, path::{Path, PathBuf}, sync::Arc};

use prjunnamed_netlist::Target;
use svql_query::{composite::SearchableComposite, netlist::SearchableNetlist, queries::security::access_control::locked_reg::{sync_en::SyncEnLockedReg, sync_mux::SyncMuxLockedReg}};
use svql_driver_handler::{proc::ProcDriver, Driver};
use svql_query::{Match, Search, WithPath};

use crate::and::AndAB;

mod and;


fn read_input(target: Option<Arc<dyn Target>>, name: String) -> Result<prjunnamed_netlist::Design, Box<dyn Error>> {
    if name.ends_with(".uir") {
        Ok(prjunnamed_netlist::parse(target, &std::fs::read_to_string(name)?)?)
    } else if name.ends_with(".json") {
        let designs = prjunnamed_yosys_json::import(target, &mut File::open(name)?)?;
        assert_eq!(designs.len(), 1, "can only convert single-module Yosys JSON to Unnamed IR");
        Ok(designs.into_values().next().unwrap())
    } else if name.is_empty() {
        panic!("no input provided")
    } else {
        panic!("don't know what to do with input {name:?}")
    }
}


fn main() {
    // env logger

    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    // let otbn_path = PathBuf::from("examples/larger_designs/otbn.json");
    // let otbn_design = read_input(None, otbn_path.to_string_lossy().to_string()).expect("Failed to read input design");


    let many_regs_path = PathBuf::from("examples/patterns/security/access_control/locked_reg/json/many_locked_regs.json");
    let many_regs_design = read_input(None, many_regs_path.to_string_lossy().to_string()).expect("Failed to read input design");


    let sync_mux_path = PathBuf::from("examples/patterns/security/access_control/locked_reg/json/sync_mux.json");
    let sync_mux_design = read_input(None, sync_mux_path.to_string_lossy().to_string()).expect("Failed to read input design");

    

    // let 

    // let cells = sync_mux_design.iter_cells_topo().collect::<Vec<_>>();
    for cell in sync_mux_design.iter_cells_topo() {
        println!("\nid: {}, cell: {:?}", cell.debug_index(), cell.get());
    }

}
