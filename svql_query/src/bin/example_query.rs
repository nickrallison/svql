use svql_query::{
    prelude::*,
    security::{cwe1234::Cwe1234, primitives::locked_register::AsyncDffMuxEnable},
};
use tracing::{Level, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let driver = Driver::new_workspace()?;
    // let mut driver = Driver::new_workspace_yosys(
    //     "/home/nick/Applications/tabby-linux-x64-latest/tabby/bin/yosys",
    // )?;

    let args = std::env::args().collect::<Vec<String>>();

    let design_path: &str = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json");

    let design_module: &str = args.get(2).map(|s| s.as_str()).unwrap_or("e203_soc_top");

    let max_recursion_depth: usize = args
        .get(3)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(2);

    let use_raw_import: bool = args
        .get(4)
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::All)
        .max_recursion_depth(Some(max_recursion_depth))
        .build();

    info!("Loading design...");
    let design_result = match use_raw_import {
        true => driver.get_or_load_design_raw(design_path, design_module),
        false => driver.get_or_load_design(design_path, design_module, &config.haystack_options),
    };

    let (haystack_key, _) = match design_result {
        Ok(res) => res,
        Err(e) => {
            info!("Could not load design (expected if file missing): {}", e);
            return Err(e.into());
        }
    };

    info!("Executing query with old API...");
    let query_results = execute_query::<Cwe1234<Search>>(&driver, &haystack_key, &config)?;
    info!("Found {} matches with old API", query_results.len());

    println!("\n=== Old API Results ===");
    for (i, match_inst) in query_results.iter().take(3).enumerate() {
        let report = match_inst.report(&format!("[Match #{}]", i + 1));
        println!("{}", report.render());
    }
    if query_results.len() > 3 {
        println!("... and {} more matches", query_results.len() - 3);
    }

    // Now test the new DataFrame API
    info!("\nExecuting query with new DataFrame API...");
    let store = svql_query::run_query::<Cwe1234<Search>>(&driver, &haystack_key)?;

    println!("\n=== New DataFrame API Results ===");
    println!("{}", store);

    if let Some(table) = store.get::<Cwe1234<Search>>() {
        println!("\n=== Cwe1234 Table Details ===");
        println!("{}", table);
    }

    if let Some(table) = store.get::<AsyncDffMuxEnable<Search>>() {
        println!("\n=== AsyncDffMuxEnable Table Details ===");
        println!("{}", table);
    }

    Ok(())
}
