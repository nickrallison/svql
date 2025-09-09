// svql_query/src/bin/run_query.rs
//
// Usage:
//   cargo run -p svql_query --bin run_query -- \
//       --query svql_query::queries::enum_composite::and_any::AndAny \
//       --haystack examples/fixtures/basic/and/json/mixed_and_tree.json \
//       --module mixed_and_tree \
//       [--dedupe auto|none] [--superset|--exact] [--progress]
//
// Notes:
// - The --query string must match a known query type discovered at build-time.
// - --module is required for the haystack (top module name for Yosys).
// - Defaults: exact-length, dedupe=none.
// - --progress shows a simple textual progress spinner while loading, then a bar when matching.

use std::env;
use std::path::PathBuf;
use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::Driver;

// Generated at build-time. Provides dispatch helpers.
mod gen_dispatch {
    include!(concat!(env!("OUT_DIR"), "/svql_query_query_dispatch.rs"));
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "\
Usage:
  run_query --query <TYPE_PATH> --haystack <PATH> --module <TOP> [--dedupe auto|none] [--superset|--exact] [--progress]

Examples:
  run_query \\
    --query svql_query::queries::netlist::basic::and::AndGate \\
    --haystack examples/fixtures/basic/and/json/mixed_and_tree.json \\
    --module mixed_and_tree \\
    --dedupe auto \\
    --progress

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
    yosys: Option<PathBuf>,
    dedupe: Dedupe,
    match_length: MatchLength,
    flatten: bool,
}

fn parse_args() -> Args {
    let mut query: Option<String> = None;
    let mut haystack: Option<String> = None;
    let mut module: Option<String> = None;
    let mut yosys: Option<PathBuf> = None;

    let mut dedupe: Option<String> = None;
    let mut match_length: Option<String> = None;
    let mut flatten: bool = false;

    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--query" | "-q" => query = it.next(),
            "--haystack" | "-H" => haystack = it.next(),
            "--module" | "-m" => module = it.next(),
            "--dedupe" => dedupe = it.next(),
            "--match_length" => match_length = it.next(),
            "--flatten" => flatten = true,
            "--yosys" => yosys = it.next().map(PathBuf::from),
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

    let dedupe: Dedupe = match dedupe.clone() {
        Some(dedupe) => dedupe.parse().unwrap_or_else(|_| {
            eprintln!("Invalid --dedupe value: {dedupe:?}");
            print_usage_and_exit();
        }),
        None => Default::default(),
    };

    let match_length: MatchLength = match match_length {
        Some(match_length) => match_length.parse().unwrap_or_else(|_| {
            eprintln!("Invalid --match_length value: {match_length:?}");
            print_usage_and_exit();
        }),
        None => Default::default(),
    };

    Args {
        query,
        haystack,
        module,
        yosys,
        dedupe,
        match_length,
        flatten,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let args = parse_args();

    let cfg = Config::builder()
        .match_length(args.match_length)
        .haystack_flatten(args.flatten)
        .dedupe(args.dedupe)
        .build();

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

    // If svql_subgraph "profiling" feature is enabled, print timings
    svql_subgraph::report();

    Ok(())
}

// fn main() {}
