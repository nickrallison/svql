use std::{path::PathBuf, sync::{Arc, Mutex}};

use svql_driver::{cache::Cache, connectivity::is_connected, get_name, subgraph::find_subgraphs, util::load_driver_cached, Driver};



fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let cache = Arc::new(Mutex::new(Cache::new()));

    let haystack_path = "examples/patterns/basic/ff/seq_sdffe.v";
    let (haystack_driver, _) = load_driver_cached(&haystack_path, cache.clone());

    let needle_path = "examples/patterns/basic/ff/sdffe.v";
    let (needle_driver, _) = load_driver_cached(&needle_path, cache.clone());

    let search_results = find_subgraphs(needle_driver.design_as_ref(), haystack_driver.design_as_ref());

    // for res in search_results.iter() {
    //     println!("Found subgraph match: {:#?}", res);
    // }

    let matches1 = search_results.matches.clone();
    let matches2 = search_results.matches.clone();

    let mut connection_count = 0;
    
    for (i, match1) in matches1.iter().enumerate() {
        for (j, match2) in matches2.iter().enumerate() {
            if i == j {
                continue; // Skip self-comparison
            }
            
            // Check if match1's "q" output connects to match2's "d" input
            if is_connected(match1, "q", match2, "d", haystack_driver.design_as_ref()) {
                println!("Found connection: match {} 'q' -> match {} 'd'", i, j);
                connection_count += 1;
            }
        }
    }


    Ok(())
}