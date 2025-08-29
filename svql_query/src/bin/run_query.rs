// svql_query/src/bin/run_query.rs
//
// Usage:
//   cargo run -p svql_query --bin run_query -- \
//       --query svql_query::queries::enum_composite::and_any::AndAny \
//       --haystack examples/fixtures/basic/and/json/mixed_and_tree.json \
//       --module mixed_and_tree \
//       [--dedupe auto|none] [--superset|--exact]
//
// Notes:
// - The --query string must match a known query type discovered at build-time.
// - --module is required for the haystack (top module name for Yosys).
// - Defaults: exact-length, dedupe=none.

use std::env;

use svql_common::{Config, DedupeMode};
use svql_driver::Driver;

// Generated at build-time. Provides:
//   - run_count_for_type_name(name, driver, haystack_path, haystack_module, &config) -> Result<usize, String>
//   - known_query_type_names() -> &'static [&'static str]
mod gen_dispatch {
    include!(concat!(env!("OUT_DIR"), "/svql_query_query_dispatch.rs"));
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "\
Usage:
  run_query --query <TYPE_PATH> --haystack <PATH> --module <TOP> [--dedupe auto|none] [--superset|--exact]

Examples:
  run_query \\
    --query svql_query::queries::enum_composite::and_any::AndAny \\
    --haystack examples/fixtures/basic/and/json/mixed_and_tree.json \\
    --module mixed_and_tree \\
    --dedupe auto

Known query types:
  {}
",
        gen_dispatch::known_query_type_names().join("\n  ")
    );
    std::process::exit(2);
}

#[derive(Debug, Clone)]
struct Args {
    query: String,
    haystack: String,
    module: String,
    dedupe: DedupeMode,
    match_length: bool, // true=exact, false=superset
}

fn parse_args() -> Args {
    let mut query: Option<String> = None;
    let mut haystack: Option<String> = None;
    let mut module: Option<String> = None;
    let mut dedupe = DedupeMode::None;
    let mut match_length = true; // exact by default

    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--query" | "-q" => {
                query = it.next();
            }
            "--haystack" | "-H" => {
                haystack = it.next();
            }
            "--module" | "-m" => {
                module = it.next();
            }
            "--dedupe" => match it.next().as_deref() {
                Some("auto") | Some("AutoMorph") | Some("automorph") => {
                    dedupe = DedupeMode::AutoMorph
                }
                Some("none") | Some("None") => dedupe = DedupeMode::None,
                other => {
                    eprintln!("Unknown dedupe mode: {:?}", other);
                    print_usage_and_exit();
                }
            },
            "--superset" => match_length = false,
            "--exact" => match_length = true,
            "--help" | "-h" => print_usage_and_exit(),
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                print_usage_and_exit();
            }
        }
    }

    let (query, haystack, module) = match (query, haystack, module) {
        (Some(q), Some(h), Some(m)) => (q, h, m),
        _ => {
            eprintln!("Missing required arguments.");
            print_usage_and_exit();
        }
    };

    Args {
        query,
        haystack,
        module,
        dedupe,
        match_length,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let args = parse_args();

    let cfg = if args.match_length {
        Config::exact_length(args.dedupe)
    } else {
        Config::superset_length(args.dedupe)
    };

    let driver = Driver::new_workspace()?;

    let count = gen_dispatch::run_count_for_type_name(
        &args.query,
        &driver,
        &args.haystack,
        &args.module,
        &cfg,
    )
    .map_err(|e| {
        format!(
            "Failed to run query '{}': {}\nKnown types:\n  {}",
            args.query,
            e,
            gen_dispatch::known_query_type_names().join("\n  ")
        )
    })?;

    println!("{}", count);

    // If profiling feature is enabled in svql_subgraph, this prints timings
    svql_subgraph::profiling::report();

    Ok(())
}
