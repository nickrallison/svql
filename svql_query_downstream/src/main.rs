use std::{cell, error::Error, fs::File, path::{Path, PathBuf}, sync::Arc};

use prjunnamed_netlist::Target;
use svql_query::{composite::SearchableComposite, netlist::SearchableNetlist, queries::security::access_control::locked_reg::{sync_en::SyncEnLockedReg, sync_mux::SyncMuxLockedReg}};
use svql_driver_handler::{proc::ProcDriver, Driver};
use svql_query::{Match, Search, WithPath};

use crate::and::AndAB;

mod and;
mod cell_index;
mod subgraph; // added

use cell_index::{CellTypeIndex, CellKind};


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


    let haystack_path = "examples/patterns/security/access_control/locked_reg/json/many_locked_regs.json";
    let haystack_design = read_input(None, haystack_path.to_string()).expect("Failed to read input design");
    let haystack_name = get_name(&haystack_path);

    let needle_path = "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
    let needle_design = read_input(None, needle_path.to_string()).expect("Failed to read input design");
    let needle_name = get_name(&needle_path);

    // Compute anchor kinds by product of gate counts across the two designs
    let anchors = cell_index::anchor_kinds_by_product(&haystack_design, &needle_design);
    println!("Anchor kinds by product (rarest first):");
    for (k, prod) in &anchors {
        println!("  {:?} -> product {}", k, prod);
    }

    let chosen_kind = if let Some((k, _)) = anchors.first() {
        *k
    } else {
        panic!("No anchor kinds found");
    };

    // Find subgraphs using the chosen anchor kind
    let matches = subgraph::find_gate_subgraphs_by_anchor_kind(&needle_design, &haystack_design, chosen_kind);
    println!("Found {} subgraph matches for {:?} (needle {} in haystack {}).", matches.len(), chosen_kind, needle_name, haystack_name);



    // Demonstrate least-common gate kind and its cells (non-gate types filtered out)
    // let index = CellTypeIndex::build(&sync_mux_design);
    // for (kind, cells) in index.iter_gate_kind_buckets_rarest_first() {
    //     println!("Rarest gate kind: {:?}", kind);
    //     for c in cells {
    //         println!("gate {:?} -> id {}", kind, c.debug_index());
    //     }
    //     // break; // only show the rarest bucket
    // }
}
