//! SVQL Pattern Matcher CLI
//!
//! Entry point for the SVQL command-line tool. Loads hardware designs and executes
//! pattern matching queries to identify security vulnerabilities or design patterns.

#![allow(unused_imports)]

#[global_allocator]
/// Global allocator using jemalloc for better performance in parallel workloads.
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod args;

use clap::Parser;
use rayon::prelude::*;
use svql_query::prelude::*;
use svql_query_lib::security::{Cwe1280, cwe1234::Cwe1234};
use tracing::info;

use args::Args;

/// Executes the SVQL pattern matcher.
///
/// This function:
/// 1. Initializes logging
/// 2. Parses command-line arguments
/// 3. Creates a design driver
/// 4. Runs a pattern matching query
/// 5. Renders and displays results
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_thread_ids(true)
        .init();

    let args = Args::parse();
    let driver = Driver::new_workspace()?;
    let config = args.to_config();
    let design_key = args.key();

    info!("Loading design: {:?}", design_key);
    info!("Executing query with DataFrame API...");

    let store = svql_query::run_query::<Cwe1280>(&driver, &design_key, &config)?;

    println!("\n=== DataFrame API Results ===");
    println!("{store}");

    for (_, table) in store.tables() {
        println!("\n=== Table Details ===");
        println!("{table}");
    }

    let rows = store
        .get::<Cwe1280>()
        .expect("Store should have table")
        .rows()
        .collect::<Vec<_>>();

    let reports = rows
        .par_iter()
        .map(|row| row.render(&store, &driver, &design_key))
        .collect::<Vec<_>>();

    for (i, report) in reports.iter().enumerate() {
        println!("{i}: {report}");
    }

    Ok(())
}
