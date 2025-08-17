use std::path::PathBuf;

use svql_driver::{get_name, Driver};



fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut cache = svql_driver::cache::Cache::new();


    let par_path = PathBuf::from("examples/patterns/basic/ff/par_double_sdffe.v");
    let par_name = get_name(par_path.to_str().unwrap());
    let par_driver = Driver::new(par_path, par_name.clone(), Some(&mut cache))?;

    let comb_path = PathBuf::from("examples/patterns/basic/ff/comb_d_double_sdffe.v");
    let comb_name = get_name(comb_path.to_str().unwrap());
    let comb_driver = Driver::new(comb_path, comb_name.clone(), Some(&mut cache))?;




    let comb_search_matches = svql_driver::subgraph::find_subgraphs(comb_driver.design_as_ref(), par_driver.design_as_ref());
    let par_search_matches = svql_driver::subgraph::find_subgraphs(par_driver.design_as_ref(), comb_driver.design_as_ref());

    println!("Comb Matches: {}", comb_search_matches.len());
    println!("Par Matches: {}", par_search_matches.len());

    assert_eq!(comb_search_matches.len(), 0, "Expected 0 matches for needle {}, against haystack {}, got {}", comb_name, par_name, comb_search_matches.len());
    assert_eq!(par_search_matches.len(), 2, "Expected 2 matches for needle {}, against haystack {}, got {}", par_name, comb_name, par_search_matches.len());

    Ok(())
}