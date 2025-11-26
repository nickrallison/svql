use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::Driver;
use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::security::cwe1280::Cwe1280;
use svql_query::traits::{Query, Searchable};
use tracing::{Level, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::None)
        .build();

    let driver = Driver::new_workspace()?;

    // Example: Load a design that might contain the vulnerability
    let design_path = "examples/fixtures/cwes/cwe1280/verilog/cwe1280_vuln.v";
    let design_module = "cwe1280_vuln";

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
    let context = Cwe1280::<Search>::context(&driver, &config.needle_options)?;
    let context = context.with_design(haystack_key.clone(), haystack_design);

    info!("Instantiating query...");
    let query = Cwe1280::<Search>::instantiate(Instance::root("cwe1280".to_string()));

    info!("Executing query...");
    let results = query.query(&driver, &context, &haystack_key, &config);

    info!("Found {} matches", results.len());

    for (i, _match) in results.iter().enumerate() {
        info!("Match #{}", i);
        // Inspect match details if needed
    }

    Ok(())
}
