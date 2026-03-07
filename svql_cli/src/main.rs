//! SVQL Pattern Matcher CLI
//!
//! Entry point for the SVQL command-line tool. Loads hardware designs and executes
//! pattern matching queries to identify security vulnerabilities or design patterns.

#![allow(unused_imports)]

#[global_allocator]
/// Global allocator using jemalloc for better performance in parallel workloads.
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod args;
mod query_registry;

use args::{Args, QueryArg};
use clap::Parser;
use query_registry::print_metrics_table;
use tracing::info;

/// Executes the SVQL pattern matcher.
///
/// This function:
/// 1. Initializes logging
/// 2. Parses command-line arguments
/// 3. Lists available queries if requested
/// 4. Creates a design driver and loads the design
/// 5. Executes selected queries with optional profiling
/// 6. Displays performance summary if profiling is enabled
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_thread_ids(true)
        .init();

    let args = Args::parse();

    // Handle list-queries flag
    if args.list_queries {
        println!("Available queries:");
        for q in QueryArg::all() {
            println!("  - {}", q.name());
        }
        println!("\nUse -q <QUERY> to run specific queries, or omit -q to run all.");
        return Ok(());
    }

    // Initialize driver
    let driver = svql_driver::Driver::new_workspace()?;
    let config = args.to_config();
    let design_key = args.key();

    info!("Loading design: {:?}", design_key);
    let container = driver.get_design(&design_key, &config.haystack_options)?;
    info!("Design loaded: {} gates", container.index().num_cells());

    // Determine which queries to run
    let queries_to_run = if args.queries.is_empty() {
        info!(
            "No specific queries selected, running all {} registered queries",
            QueryArg::all().len()
        );
        QueryArg::all()
    } else {
        args.queries.clone()
    };

    info!(
        "Executing {} query(ies) with parallel={}",
        queries_to_run.len(),
        args.parallel
    );

    // Execute queries and collect metrics
    let mut all_metrics = Vec::new();
    let mut had_errors = false;

    for query in queries_to_run {
        info!("Starting query: {}", query.name());

        match query.run(
            &driver,
            &design_key,
            &config,
            args.profile,
            args.print_results,
        ) {
            Ok(metrics) => {
                all_metrics.push(metrics);
            }
            Err(e) => {
                eprintln!("Error executing query {}: {}", query.name(), e);
                had_errors = true;
            }
        }
    }

    // Print profiling summary table
    if args.profile && !all_metrics.is_empty() {
        print_metrics_table(&all_metrics);

        // Additional summary statistics
        let total_matches: usize = all_metrics.iter().map(|m| m.matches).sum();
        let total_time: std::time::Duration = all_metrics
            .iter()
            .filter_map(|m| m.perf.as_ref())
            .map(|p| p.duration)
            .sum();
        println!("\nTotal matches across all queries: {}", total_matches);
        println!("Total execution time: {:.2?}", total_time);
    }

    // Print detailed results if requested
    if args.print_results && !all_metrics.is_empty() {
        query_registry::print_all_results(&all_metrics, &driver, &design_key, &config);
    }

    if had_errors {
        eprintln!("\nWarning: Some queries failed to execute. Check logs above.");
    }

    Ok(())
}
