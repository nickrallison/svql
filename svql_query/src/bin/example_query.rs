use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::Driver;
use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::ir::NaiveExecutor;
use svql_query::security::cwe1234::Cwe1234;
use svql_query::traits::{PlannedQuery, Query, Searchable};
use tracing::{Level, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::None)
        .max_recursion_depth(Some(3))
        .build();

    let driver = Driver::new_workspace()?;

    // let design_path = "examples/fixtures/larger_designs/json/openpiton_tile_flat.json";
    // let design_module = "tile";

    let design_path =
        "examples/fixtures/larger_designs/json/hackatdac18/cv32e40p_fp_wrapper_netlist.json";
    let design_module = "cv32e40p_fp_wrapper";

    info!("Loading design...");
    let (haystack_key, haystack_design) =
        match driver.get_or_load_design(design_path, design_module, &config.haystack_options) {
            Ok(res) => res,
            Err(e) => {
                info!("Could not load design (expected if file missing): {}", e);
                return Ok(());
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

    for (i, match_) in query_results.iter().enumerate() {
        info!("Match #{}: {:#?}", i, match_);
        // Inspect match details if needed
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
