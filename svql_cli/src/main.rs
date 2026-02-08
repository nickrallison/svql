#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod args;

use clap::Parser;
use svql_query::prelude::*;
use svql_query_lib::security::cwe1234::Cwe1234;
use tracing::info;

use args::Args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_thread_ids(true)
        .init();

    // Parse command-line arguments
    let args = Args::parse();

    let driver = Driver::new_workspace()?;

    let config = args.to_config();

    let design_key = args.key();
    info!("Loading design: {:?}", design_key);

    info!("Executing query with DataFrame API...");
    let store = svql_query::run_query::<Cwe1234>(&driver, &design_key, &config)?;

    println!("\n=== DataFrame API Results ===");
    println!("{store}");

    for (_, table) in store.tables() {
        println!("\n=== Table Details ===");
        println!("{table}");
    }

    for row in store
        .get::<Cwe1234>()
        .expect("Store should have table")
        .rows()
        .take(20)
    {
        let report = row.render(&store, &driver, &design_key);
        println!("{}", report);
    }

    Ok(())
}
