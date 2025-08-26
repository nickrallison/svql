// svql_query/src/bin/and_any_otbn.rs

use svql_common::{Config, DedupeMode};
use svql_driver::Driver;
use svql_query::WithPath;
use svql_query::composite::SearchableEnumComposite;
use svql_query::instance::Instance;
use svql_query::queries::enum_composite::and_any::AndAny;
use tracing_subscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    println!("Running AndAny query on OTBN design...");

    // Create driver for the workspace
    let driver = Driver::new_workspace()?;

    // Create context for the AndAny query (loads all AND pattern variants)
    let context = AndAny::<svql_query::Search>::context(&driver)?;

    // Load the OTBN design as haystack
    let otbn_path = "examples/fixtures/larger_designs/json/otbn_core.json";
    let otbn_module = "otbn_core";

    let (haystack_key, haystack_design) =
        driver.get_or_load_design(otbn_path, otbn_module.to_string())?;

    // Add the OTBN design to the context
    let context = context.with_design(haystack_key.clone(), haystack_design);

    // Configure the query
    let config = Config::builder()
        .exact_length() // Match exact pin counts
        .dedupe(DedupeMode::AutoMorph) // Deduplicate by automorphism
        .build();

    // Create root instance path for results
    let root = Instance::root("otbn_and_any_query".to_string());

    // Run the AndAny query on the OTBN design
    let hits = AndAny::<svql_query::Search>::query(&haystack_key, &context, root, &config);

    // Report results
    println!("Found {} AND gate instances in OTBN design:", hits.len());

    let mut gate_count = 0;
    let mut mux_count = 0;
    let mut nor_count = 0;

    for hit in &hits {
        match hit {
            AndAny::Gate(_) => gate_count += 1,
            AndAny::Mux(_) => mux_count += 1,
            AndAny::Nor(_) => nor_count += 1,
        }
    }

    println!("  - AndGate instances: {}", gate_count);
    println!("  - AndMux instances: {}", mux_count);
    println!("  - AndNor instances: {}", nor_count);

    // You could also iterate through hits to examine specific matches
    for (i, hit) in hits.iter().enumerate() {
        // For enum variants, we need to get the path from each variant
        let path_str = match hit {
            AndAny::Gate(g) => g.path().inst_path(),
            AndAny::Mux(m) => m.path().inst_path(),
            AndAny::Nor(n) => n.path().inst_path(),
        };
        println!("Match {}: {}", i, path_str);
    }

    Ok(())
}
