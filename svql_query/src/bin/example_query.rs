use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::Driver;
use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::ir::NaiveExecutor;
use svql_query::security::cwe1234::Cwe1234;
use svql_query::traits::{PlannedQuery, Searchable};
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

    let design_path = "examples/fixtures/cwes/cwe1234/cwe1234_not_alternating.v";
    let design_module = "cwe1234_not_alternating";

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
    let executor = NaiveExecutor;
    let results = query.query_planned(&executor, &context, &haystack_key, &config);
    // let results = query.query(&driver, &context, &haystack_key, &config);

    info!("Found {} matches", results.len());

    for (i, _match) in results.iter().enumerate() {
        info!("Match #{}", i);
        // Inspect match details if needed
    }

    Ok(())
}
