use std::path::PathBuf;

use svql_driver::{get_name, Driver};



fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut cache = svql_driver::cache::Cache::new();

    let sep_path = PathBuf::from("examples/patterns/basic/ff/par_sep_double_sdffe.v");
    let sep_name = get_name(sep_path.to_str().unwrap());
    let sep_driver = Driver::new(sep_path, sep_name.clone(), &mut cache)?;

    let comb_path = PathBuf::from("examples/patterns/basic/ff/par_combined_double_sdffe.v");
    let comb_name = get_name(comb_path.to_str().unwrap());
    let comb_driver = Driver::new(comb_path, comb_name.clone(), &mut cache)?;

    let comb_search_matches = svql_driver::subgraph::find_subgraphs(comb_driver.design_as_ref(), sep_driver.design_as_ref());
    let sep_search_matches = svql_driver::subgraph::find_subgraphs(sep_driver.design_as_ref(), comb_driver.design_as_ref());

    println!("Comb Matches: {}", comb_search_matches.len());
    println!("Sep Matches: {}", sep_search_matches.len());

    assert_eq!(comb_search_matches.len(), 0, "Expected 0 matches for needle {}, against haystack {}, got {}", comb_name, sep_name, comb_search_matches.len());
    assert_eq!(sep_search_matches.len(), 2, "Expected 2 matches for needle {}, against haystack {}, got {}", sep_name, comb_name, sep_search_matches.len());

    Ok(())
}