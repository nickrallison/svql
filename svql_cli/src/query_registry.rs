//! Query registration and profiling infrastructure.
//!
//! Provides the `register_queries!` macro to expose query types to the CLI
//! with integrated timing and memory profiling.

use std::any::TypeId;
use std::time::{Duration, Instant};
use svql_query::session::Store;
use sysinfo::{System, get_current_pid};

/// Performance metrics collected during query execution.
#[derive(Debug, Clone)]
pub struct PerfMetrics {
    /// Execution duration
    pub duration: Duration,
    /// Memory delta in MB
    pub memory_delta_mb: f64,
}

/// Metrics and results collected during query execution.
#[derive(Debug)]
pub struct QueryMetrics {
    /// Name of the query
    pub name: &'static str,
    /// Number of matches found (cached for convenience)
    pub matches: usize,
    /// Optional performance metrics (None if profiling disabled)
    pub perf: Option<PerfMetrics>,
    /// The actual query results store containing match data
    pub store: Option<Store>,
    /// TypeId of the root query pattern (to identify which table is the primary result)
    pub root_type_id: TypeId,
}

/// Macro to register available queries and generate CLI enum.
#[macro_export]
macro_rules! register_queries {
    ($enum_name:ident { $($variant:ident => $type:ty),* $(,)? }) => {
        #[derive(Debug, Clone, ::clap::ValueEnum)]
        pub enum $enum_name {
            $(
                $variant,
            )*
        }

        impl $enum_name {
            /// Get all available query variants.
            pub fn all() -> Vec<Self> {
                vec![
                    $(Self::$variant,)*
                ]
            }

            /// Get the name of this query.
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)*
                }
            }

            /// Run the query with optional profiling and return metrics with results.
            ///
            /// # Arguments
            /// * `driver` - The design driver
            /// * `key` - The design key
            /// * `config` - Query configuration
            /// * `enable_profiling` - Whether to collect timing/memory metrics
            pub fn run(
                &self,
                driver: &::svql_driver::Driver,
                key: &::svql_driver::DriverKey,
                config: &::svql_common::Config,
                enable_profiling: bool,
            ) -> Result<$crate::query_registry::QueryMetrics, Box<dyn std::error::Error>> {
                use ::std::time::Instant;
                use ::sysinfo::System;
                use ::std::any::TypeId;

                let mut sys = System::new_all();

                // Get current PID - handle the Result properly
                let pid = ::sysinfo::get_current_pid()
                    .map_err(|e| format!("Failed to get current PID: {}", e))?;

                // Initial memory measurement
                let initial_mem = if enable_profiling {
                    sys.refresh_all();
                    sys.process(pid)
                        .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                let start = Instant::now();

                // Execute the appropriate query type
                let store = match self {
                    $(Self::$variant => {
                        ::svql_query::run_query::<$type>(driver, key, config)?
                    },)*
                };

                let duration = start.elapsed();

                // Final memory measurement
                let final_mem = if enable_profiling {
                    sys.refresh_all();
                    sys.process(pid)
                        .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                // Extract match count
                let matches = match self {
                    $(Self::$variant => {
                        use ::svql_query::session::Store;
                        store.get::<$type>().map(|t| t.len()).unwrap_or(0)
                    },)*
                };

                // Build performance metrics if enabled
                let perf = if enable_profiling {
                    // Use f64::max for float subtraction to avoid negative values
                    let memory_delta = (final_mem - initial_mem).max(0.0);
                    Some($crate::query_registry::PerfMetrics {
                        duration,
                        memory_delta_mb: memory_delta,
                    })
                } else {
                    None
                };

                // Output results
                println!("\n=== Results for {} ===", self.name());
                println!("Matches found: {}", matches);

                if let Some(ref p) = perf {
                    println!("Execution time: {:.2?}", p.duration);
                    println!("Memory delta: {:.2} MB", p.memory_delta_mb);
                }

                // Render detailed results for small match sets
                if matches > 0 && matches <= 5 {
                    match self {
                        $(Self::$variant => {
                            use ::svql_query::traits::Pattern;
                            if let Some(table) = store.get::<$type>() {
                                for (i, (_, row)) in table.rows().enumerate() {
                                    println!("\n--- Match #{} ---", i);
                                    let report = <$type as Pattern>::render_row(
                                        &row, &store, driver, key, config
                                    );
                                    println!("{}", report);
                                }
                            }
                        })*
                    }
                } else if matches > 5 {
                    println!("({} matches found, use --profile for summary)", matches);
                }

                Ok($crate::query_registry::QueryMetrics {
                    name: self.name(),
                    matches,
                    perf,
                    store: Some(store),
                    root_type_id: match self {
                        $(Self::$variant => TypeId::of::<$type>(),)*
                    },
                })
            }
        }
    };
}

/// Print a formatted summary table of query metrics.
pub fn print_metrics_table(metrics: &[QueryMetrics]) {
    if metrics.is_empty() {
        return;
    }

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                    SVQL Query Performance Summary                    ║");
    println!("╠══════════════════════════════════════════════════════════════════════╣");

    // Check if we have any perf metrics to display
    let has_perf = metrics.iter().any(|m| m.perf.is_some());

    if has_perf {
        println!(
            "║ {:<20} {:>10} {:>15} {:>18} ║",
            "Query", "Matches", "Time (ms)", "Memory (MB)"
        );
    } else {
        println!("║ {:<20} {:>10} {:>30} ║", "Query", "Matches", "Status");
    }
    println!("╠══════════════════════════════════════════════════════════════════════╣");

    for m in metrics {
        if let Some(ref p) = m.perf {
            let time_ms = p.duration.as_millis();
            println!(
                "║ {:<20} {:>10} {:>15} {:>18.2} ║",
                m.name, m.matches, time_ms, p.memory_delta_mb
            );
        } else {
            println!("║ {:<20} {:>10} {:>30} ║", m.name, m.matches, "Completed");
        }
    }

    println!("╚══════════════════════════════════════════════════════════════════════╝");

    // Print aggregate statistics if we have perf data
    let perf_metrics: Vec<_> = metrics.iter().filter_map(|m| m.perf.as_ref()).collect();
    if !perf_metrics.is_empty() {
        let total_matches: usize = metrics.iter().map(|m| m.matches).sum();
        let total_time: Duration = perf_metrics.iter().map(|p| p.duration).sum();
        let total_memory: f64 = perf_metrics.iter().map(|p| p.memory_delta_mb).sum();

        println!("\nAggregate Statistics:");
        println!("  Total matches: {}", total_matches);
        println!("  Total execution time: {:.2?}", total_time);
        println!("  Total memory delta: {:.2} MB", total_memory);
        println!(
            "  Average time per query: {:.2?}",
            total_time / perf_metrics.len() as u32
        );
    }
}

/// Print detailed results for all queries that have matches.
/// Only prints the root query results (highest level), not sub-components.
pub fn print_all_results(
    metrics: &[QueryMetrics],
    driver: &svql_driver::Driver,
    key: &svql_driver::DriverKey,
    config: &svql_common::Config,
) {
    for metric in metrics {
        if let Some(ref store) = metric.store {
            if metric.matches == 0 {
                continue;
            }

            println!("\n╔══════════════════════════════════════════════════════════════════════╗");
            println!("║ Results for: {:<56} ║", metric.name);
            println!("╚══════════════════════════════════════════════════════════════════════╝");

            // Only print the root query table, not all sub-component tables
            if let Some(table) = store.get_any(metric.root_type_id) {
                print_query_table(table, store, driver, key, config);
            }
        }
    }
}

/// Print a single query table with limited output.
fn print_query_table(
    table: &dyn svql_query::session::AnyTable,
    store: &Store,
    driver: &svql_driver::Driver,
    key: &svql_driver::DriverKey,
    _config: &svql_common::Config,
) {
    if table.is_empty() {
        return;
    }

    println!("--- {} ({} matches) ---", table.type_name(), table.len());

    // Print first few matches (limit to avoid spam)
    let limit = 10;
    for (i, row_idx) in (0..table.len()).enumerate() {
        if i >= limit {
            println!("... and {} more matches", table.len() - limit);
            break;
        }

        if let Some(node) = table.row_to_report_node(row_idx, store, driver, key) {
            println!("{}", node.render());
        }
    }
}

/// Access results from a specific query by name.
#[allow(dead_code)]
pub fn get_query_results<'a>(metrics: &'a [QueryMetrics], name: &str) -> Option<&'a Store> {
    metrics
        .iter()
        .find(|m| m.name == name)
        .and_then(|m| m.store.as_ref())
}
