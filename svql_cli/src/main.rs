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
use query_registry::{export_csv, print_metrics_table};
use tracing::info;

/// Executes the SVQL pattern matcher.
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
        println!(
            "Use -d <SPEC> to specify designs. Format: path.json,module,[raw|no-raw],[first|subset|exact]"
        );
        return Ok(());
    }

    if args.designs.is_empty() {
        eprintln!("Error: No designs specified. Use -d to specify at least one design.");
        std::process::exit(1);
    }

    info!(
        "Running {} design(s) with global parallel={}",
        args.designs.len(),
        args.parallel
    );

    // Initialize driver
    let driver = svql_driver::Driver::new_workspace()?;

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

    // Execute queries against all designs with their individual configs
    let mut all_metrics = Vec::new();
    let mut had_errors = false;

    for design_input in &args.designs {
        let key = design_input.key();
        let design_config = design_input.to_config(args.parallel);

        info!(
            "Loading design: {} (module: {}, raw: {}, match: {:?})",
            design_input.path, design_input.module, design_input.use_raw, design_input.match_length
        );

        let container = match driver.get_design(&key, &design_config.haystack_options) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading design {}: {}", key.module_name(), e);
                had_errors = true;
                continue;
            }
        };

        info!("Design loaded: {} gates", container.index().num_cells());

        for query in &queries_to_run {
            info!("Starting query: {} on {}", query.name(), key.module_name());

            match query.run(
                &driver,
                &key,
                &design_config, // Use design-specific config
                args.profile,
                args.print_results,
                design_input.use_raw,
                &design_input.match_length,
            ) {
                Ok(metrics) => {
                    all_metrics.push(metrics);
                }
                Err(e) => {
                    eprintln!(
                        "Error executing query {} on {}: {}",
                        query.name(),
                        key.module_name(),
                        e
                    );
                    had_errors = true;
                }
            }
        }
    }

    // Print profiling summary table
    if args.profile && !all_metrics.is_empty() {
        print_metrics_table(&all_metrics);

        let total_matches: usize = all_metrics.iter().map(|m| m.matches).sum();
        let total_time: std::time::Duration = all_metrics
            .iter()
            .filter_map(|m| m.perf.as_ref())
            .map(|p| p.duration)
            .sum();
        println!(
            "\nTotal matches across all queries and designs: {}",
            total_matches
        );
        println!("Total execution time: {:.2?}", total_time);
    }

    // Print detailed results if requested
    if args.print_results && !all_metrics.is_empty() {
        println!("\n=== Detailed Results ===");
        for metric in &all_metrics {
            if let Some(ref store) = metric.store {
                if metric.matches == 0 {
                    continue;
                }

                println!(
                    "\n--- {} on {} ({} matches) ---",
                    metric.name, metric.design_module, metric.matches
                );

                if let Some(table) = store.get_any(metric.root_type_id) {
                    // Create a dummy key for display - in production you'd store the key in metrics
                    let key =
                        svql_driver::DriverKey::new(&metric.design_path, &metric.design_module);
                    if let Some(node) = table.row_to_report_node(0, store, &driver, &key) {
                        println!("{}", node.render());
                        if table.len() > 1 {
                            println!("... and {} more matches", table.len() - 1);
                        }
                    }
                }
            }
        }
    }

    // Export CSV if requested
    if let Some(csv_path) = args.output_csv {
        info!("Exporting results to CSV: {}", csv_path);
        export_csv(&all_metrics, &csv_path)?;
        println!("Results exported to: {}", csv_path);
    }

    if had_errors {
        eprintln!("\nWarning: Some queries or designs failed to execute. Check logs above.");
        std::process::exit(1);
    }

    Ok(())
}
