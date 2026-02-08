#![allow(unused_imports)]

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod args;

use clap::Parser;
use rayon::prelude::*;
use svql_query::prelude::*;
use svql_query_lib::security::{
    cwe1234::Cwe1234, locked_register::LockedRegister, unlock_logic::UnlockLogic,
};
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

    let rows = store
        .get::<UnlockLogic>()
        .expect("Store should have table")
        .rows()
        .collect::<Vec<_>>();

    let filter = |report: &str| -> bool {
        // report.contains("CellId: 900")
        true
    };

    let reports = rows
        .par_iter()
        .map(|row| row.render(&store, &driver, &design_key))
        .filter(|report| filter(report))
        .collect::<Vec<_>>();

    for (i, report) in reports.iter().enumerate() {
        // if report.contains
        println!("{i}: {report}");
    }

    store.to_csv_dir("csvs")?;

    Ok(())
}
