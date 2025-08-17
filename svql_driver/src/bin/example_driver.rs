use std::path::PathBuf;

use svql_driver::{get_name, read_input_to_design, Driver};



fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut cache = svql_driver::cache::Cache::new();


    let haystack_path = PathBuf::from("examples/larger_designs/otbn_core.json");
    let haystack_name = get_name(haystack_path.to_str().unwrap());
    let haystack_driver = Driver::new(haystack_path, haystack_name.clone(), &mut cache)?;

    let needle_path = PathBuf::from("examples/patterns/security/access_control/locked_reg/json/async_en.json");
    let needle_name = get_name(needle_path.to_str().unwrap());
    let needle_driver = Driver::new(needle_path, needle_name.clone(), &mut cache)?;

    // Find subgraphs using the chosen anchor kind
    let matches = svql_driver::subgraph::find_subgraphs(needle_driver.design_as_ref(), haystack_driver.design_as_ref());
    // assert_eq!(matches.len(), 207, "Expected 207 matches for needle {}, against haystack {}, got {}", needle_name, haystack_name, matches.len());

    println!("Found {} matches for needle '{}' against haystack '{}'", matches.len(), needle_name, haystack_name);

    // for (i, match_map) in matches.iter().enumerate() {
    //     println!("Match {} ({} pairs):", i + 1, match_map.len());
    //     for (needle_cell_ref, design_cell_ref) in match_map.iter() {
    //         let needle_meta = needle_cell_ref.metadata();
    //         let design_meta = design_cell_ref.metadata();
    //         println!("Needle Cell Meta: {:?}, \nDesign Cell Meta: {:?}\n---\n", needle_meta.get(), design_meta.get());

    //         let needle_cell = needle_cell_ref.get();
    //         let design_cell = design_cell_ref.get();

    //         println!("Needle Cell: {:?}, \nDesign Cell: {:?}\n---\n", needle_cell, design_cell);
    //     }
    // }

    Ok(())
}