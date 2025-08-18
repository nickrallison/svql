
use std::sync::{Arc, Mutex};

use svql_driver::{cache::Cache, subgraph::find_subgraphs, util::{load_driver_cached}};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let cache = Arc::new(Mutex::new(Cache::new()));

    let haystack_path = "examples/patterns/basic/ff/seq_double_sdffe.v";
    let (haystack_driver, _) = load_driver_cached(&haystack_path, cache.clone());

    let needle_path = "examples/patterns/basic/ff/sdffe.v";
    let (needle_driver, _) = load_driver_cached(&needle_path, cache.clone());

    let search_results = find_subgraphs(needle_driver.design_as_ref(), haystack_driver.design_as_ref());
    for res in search_results.iter() {
        println!("Found subgraph match: {:#?}", res);
    }
    // assert_eq!(search_results.len(), 207, "Expected 207 matches for needle {}, against haystack {}, got {}", needle_name, haystack_name, search_results.len());

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