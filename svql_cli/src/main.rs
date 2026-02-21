//! SVQL Pattern Matcher CLI
//!
//! Entry point for the SVQL command-line tool. Loads hardware designs and executes
//! pattern matching queries to identify security vulnerabilities or design patterns.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod args;

pub mod step1;
pub mod step2;
pub mod step3;
pub mod step4;
pub mod step5;

use clap::Parser;
use rayon::prelude::*;
use svql_query::prelude::*;
use tracing::info;

use crate::step1::AdcWithCarry;
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

    let store = svql_query::run_query::<AdcWithCarry>(&driver, &design_key, &config)?;

    println!("\n=== DataFrame API Results ===");
    println!("{store}");

    for (_, table) in store.tables() {
        println!("\n=== Table Details ===");
        println!("{table}");
    }

    let rows = store
        .get::<AdcWithCarry>()
        .expect("Store should have table")
        .rows()
        .collect::<Vec<_>>();

    let reports = rows
        .par_iter()
        .map(|(_ref, row)| row.render(&store, &driver, &design_key, &config))
        .collect::<Vec<_>>();

    for (i, report) in reports.iter().enumerate() {
        println!("{i}: {report}");
    }

    Ok(())
}
