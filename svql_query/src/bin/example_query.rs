use common::{Config, Dedupe, MatchLength};
use driver::Driver;
use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::security::cwe1234::Cwe1234;
use svql_query::traits::Reportable;
use svql_query::traits::{Query, Searchable};
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

    let (haystack_key, haystack_design) = match design_result {
        Ok(res) => res,
        Err(e) => {
            info!("Could not load design (expected if file missing): {}", e);
            return Err(e.into());
        }
    };

    info!("Building context...");
    let context = Cwe1234::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    info!("Instantiating query...");
    let query = Cwe1234::<Search>::instantiate(Instance::root("cwe1234".to_string()));

    info!("Executing query...");
    // let executor = NaiveExecutor;
    // let planned_query_results = query.query_planned(&executor, &context, &haystack_key, &config);
    let query_results = query.query(&driver, &context, &haystack_key, &config);

    info!("Found {} matches for old query", query_results.len());

    for (i, match_inst) in query_results.iter().enumerate() {
        let report = match_inst.to_report(&format!("[Match #{}]", i + 1));
        println!("{}", report.render());
    }

    // info!(
    //     "Found {} matches for planned query",
    //     planned_query_results.len()
    // );
    // for (i, _match) in planned_query_results.iter().enumerate() {
    //     info!("Planned Match #{}", i);
    //     // Inspect match details if needed
    // }

    Ok(())
}
