use std::path::PathBuf;

use svql_driver::{get_name, read_input_to_design, Driver};
use svql_query::{instance::Instance, netlist::SearchableNetlist, queries::basic::and::And, Search};



fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut cache = svql_driver::cache::Cache::new();


    let haystack_path = PathBuf::from("examples/larger_designs/otbn_core.json");
    let haystack_name = get_name(haystack_path.to_str().unwrap());
    let haystack_driver = Driver::new(haystack_path, haystack_name.clone(), &mut cache)?;

    let needle_path = PathBuf::from("examples/patterns/basic/and/and.v");
    let needle_name = "and_gate".to_string(); //get_name(needle_path.to_str().unwrap());
    let needle_driver = Driver::new(needle_path, needle_name.clone(), &mut cache)?;

    let search_results = And::<Search>::query(&needle_driver, &haystack_driver, Instance::root("and".to_string()));
    assert_eq!(search_results.len(), 207, "Expected 207 matches for needle {}, against haystack {}, got {}", needle_name, haystack_name, search_results.len());

    // for (i, match_map) in search_results.iter().enumerate() {
    //     println!("Match {} ({} pairs):", i + 1, match_map.len());
    //     for (needle_cell_ref, design_cell_ref) in match_map.iter() {
    //         let needle_meta = needle_cell_ref.metadata();
    //         let design_meta = design_cell_ref.metadata();
    //         println!("Needle Cell: {:?}, \nDesign Cell: {:?}\n---\n", needle_meta.get(), design_meta.get());
    //     }
    // }

    Ok(())
}