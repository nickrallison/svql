// svql_query/src/bin/example_query.rs
//
use std::env;
use std::path::PathBuf;
use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::Driver;
use svql_query::{
    composite::SearchableEnumComposite, instance::Instance,
    queries::enum_composite::and_any::AndAny,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let cfg = Config::builder()
        .match_length(MatchLength::First)
        .dedupe(Dedupe::None)
        .build();

    let design_path = "examples/fixtures/larger_designs/json/openpiton_tile_full.json";
    let design_module = "tile";

    let driver = Driver::new_workspace()?;
    let (design_key, design_arc) =
        driver.get_or_load_design(design_path, design_module, &cfg.haystack_options)?;

    let context = AndAny::context(&driver, &cfg.haystack_options)?;
    let context = context.with_design(design_key.clone(), design_arc);

    let and_any_results = AndAny::query(
        &design_key,
        &context,
        Instance::root("and_any".to_string()),
        &cfg,
    );

    let count = and_any_results.len();

    println!("{}", count);

    Ok(())
}

// fn main() {}
