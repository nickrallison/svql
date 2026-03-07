//! Query registration and profiling infrastructure.
//!
//! Provides the `register_queries!` macro to expose query types to the CLI
//! with integrated timing and memory profiling.

use std::any::TypeId;
use std::path::Path;
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
    /// Design path
    pub design_path: String,
    /// Design module
    pub design_module: String,
    /// Whether raw import was used for this design
    pub use_raw: bool,
    /// Match length strategy used
    pub match_length: String,
    /// Number of matches found (cached for convenience)
    pub matches: usize,
    /// Optional performance metrics (None if profiling disabled)
    pub perf: Option<PerfMetrics>,
    /// The actual query results store containing match data
    pub store: Option<Store>,
    /// TypeId of the root query pattern (to identify which table is the primary result)
    pub root_type_id: TypeId,
}

impl QueryMetrics {
    /// Get execution time in milliseconds, or None if profiling disabled
    pub fn time_ms(&self) -> Option<u128> {
        self.perf.as_ref().map(|p| p.duration.as_millis())
    }

    /// Get memory delta in MB, or None if profiling disabled
    pub fn memory_mb(&self) -> Option<f64> {
        self.perf.as_ref().map(|p| p.memory_delta_mb)
    }
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
            /// * `config` - Query configuration (design-specific)
            /// * `enable_profiling` - Whether to collect timing/memory metrics
            /// * `print_results` - Whether detailed results will be printed later (for hint message)
            pub fn run(
                &self,
                driver: &::svql_driver::Driver,
                key: &::svql_driver::DriverKey,
                config: &::svql_common::Config,
                enable_profiling: bool,
                print_results: bool,
                use_raw: bool,
                match_length: &$crate::args::MatchLengthArg,
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

                // Output basic summary (always printed)
                println!("\n=== Results for {} on {} ===", self.name(), key.module_name());
                println!("Matches found: {}", matches);

                if let Some(ref p) = perf {
                    println!("Execution time: {:.2?}", p.duration);
                    println!("Memory delta: {:.2} MB", p.memory_delta_mb);
                }

                // Print hint about using --print-results if there are matches and we're not printing them now
                if !print_results && matches > 0 {
                    println!("({} matches found, use --print-results to display details)", matches);
                }

                Ok($crate::query_registry::QueryMetrics {
                    name: self.name(),
                    design_path: key.path().to_string_lossy().to_string(),
                    design_module: key.module_name().to_string(),
                    use_raw,
                    match_length: format!("{:?}", match_length),
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

    println!(
        "\n╔══════════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║                           SVQL Query Performance Summary                             ║"
    );
    println!(
        "╠══════════════════════════════════════════════════════════════════════════════════════╣"
    );

    // Check if we have any perf metrics to display
    let has_perf = metrics.iter().any(|m| m.perf.is_some());

    if has_perf {
        println!(
            "║ {:<12} {:<15} {:<8} {:>10} {:>12} {:>12} ║",
            "Query", "Design", "Config", "Matches", "Time(ms)", "Mem(MB)"
        );
    } else {
        println!(
            "║ {:<12} {:<15} {:<8} {:>10} {:>30} ║",
            "Query", "Design", "Config", "Matches", "Status"
        );
    }
    println!(
        "╠══════════════════════════════════════════════════════════════════════════════════════╣"
    );

    for m in metrics {
        let design_short = if m.design_module.len() > 15 {
            format!("{}...", &m.design_module[..12])
        } else {
            m.design_module.clone()
        };

        let config_short = format!(
            "{}{}",
            if m.use_raw { "R" } else { "Y" },
            match m.match_length.as_str() {
                "First" => "F",
                "NeedleSubsetHaystack" => "S",
                "Exact" => "E",
                _ => "?",
            }
        );

        if let Some(ref p) = m.perf {
            let time_ms = p.duration.as_millis();
            println!(
                "║ {:<12} {:<15} {:<8} {:>10} {:>12} {:>12.2} ║",
                m.name, design_short, config_short, m.matches, time_ms, p.memory_delta_mb
            );
        } else {
            println!(
                "║ {:<12} {:<15} {:<8} {:>10} {:>30} ║",
                m.name, design_short, config_short, m.matches, "Completed"
            );
        }
    }

    println!(
        "╚══════════════════════════════════════════════════════════════════════════════════════╝"
    );

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

/// Export metrics to CSV file.
pub fn export_csv<P: AsRef<Path>>(
    metrics: &[QueryMetrics],
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = csv::Writer::from_path(path)?;

    // Write header
    writer.write_record(&[
        "design_path",
        "design_module",
        "use_raw_import",
        "match_length",
        "query_name",
        "matches",
        "execution_time_ms",
        "memory_delta_mb",
    ])?;

    // Write data
    for m in metrics {
        let raw = if m.use_raw {
            "true".to_string()
        } else {
            "false".to_string()
        };
        writer.write_record(&[
            &m.design_path,
            &m.design_module,
            &raw,
            &m.match_length,
            m.name,
            &m.matches.to_string(),
            &m.time_ms().map(|t| t.to_string()).unwrap_or_default(),
            &m.memory_mb()
                .map(|m| format!("{:.2}", m))
                .unwrap_or_default(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

/// Print detailed results for all queries that have matches.
/// Only prints the root query results (highest level), not sub-components.
pub fn print_all_results(
    metrics: &[QueryMetrics],
    driver: &svql_driver::Driver,
    config: &svql_common::Config,
) {
    for metric in metrics {
        if let Some(ref store) = metric.store {
            if metric.matches == 0 {
                continue;
            }

            println!("\n╔══════════════════════════════════════════════════════════════════════╗");
            println!(
                "║ Results for: {:<20} on {:<30} ║",
                metric.name, metric.design_module
            );
            println!("╚══════════════════════════════════════════════════════════════════════╝");

            // Only print the root query table, not all sub-component tables
            if let Some(table) = store.get_any(metric.root_type_id) {
                print_query_table(table, store, driver, config);
            }
        }
    }
}

/// Print a single query table with limited output.
fn print_query_table(
    table: &dyn svql_query::session::AnyTable,
    store: &Store,
    driver: &svql_driver::Driver,
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

        // Need the design key for this metric - simplified version
        // In real usage, we'd need to pass the key through
        let key = svql_driver::DriverKey::new("", ""); // Placeholder - should be fixed

        if let Some(node) = table.row_to_report_node(row_idx, store, driver, &key) {
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
