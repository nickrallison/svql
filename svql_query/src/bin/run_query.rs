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
use std::io::{self, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use svql_common::{Config, DedupeMode};
use svql_driver::Driver;
use svql_subgraph::progress::Progress;

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
    dedupe: DedupeMode,
    match_length: bool, // true=exact, false=superset
    show_progress: bool,
}

fn parse_args() -> Args {
    let mut query: Option<String> = None;
    let mut haystack: Option<String> = None;
    let mut module: Option<String> = None;
    let mut dedupe = DedupeMode::None;
    let mut match_length = true; // exact by default
    let mut show_progress = false;

    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--query" | "-q" => query = it.next(),
            "--haystack" | "-H" => haystack = it.next(),
            "--module" | "-m" => module = it.next(),
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
            "--progress" | "-p" => show_progress = true,
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
        show_progress,
    }
}

fn render_progress_bar(s: svql_subgraph::progress::ProgressSnapshot) -> String {
    let total = s.total_candidates.max(1); // avoid div-by-zero if used standalone
    let scanned = s.scanned_candidates.min(total);
    let pct = (scanned as f64 / total as f64) * 100.0;

    let width = 40usize;
    let filled = ((scanned as f64 / total as f64) * width as f64).round() as usize;
    let filled = filled.min(width);

    let mut bar = String::with_capacity(width + 2);
    bar.push('[');
    for _ in 0..filled {
        bar.push('#');
    }
    for _ in filled..width {
        bar.push('.');
    }
    bar.push(']');

    format!("{} {:>10}/{:<10} ({:>5.1}%)", bar, scanned, total, pct)
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

    if args.show_progress {
        // Progress shared with the matching engine (once it starts).
        let progress = Arc::new(Progress::new());
        // Spinner/renderer stop flag.
        let stop = Arc::new(AtomicBool::new(false));

        let p_for_thread = progress.clone();
        let stop_for_thread = stop.clone();

        // Background renderer: spinner while total==0 (design loading), progress bar once search begins.
        let handle = thread::spawn(move || {
            let mut first = true;
            let frames: [char; 4] = ['|', '/', '-', '\\'];
            let mut fi = 0usize;

            while !stop_for_thread.load(Ordering::Relaxed) {
                let snap = p_for_thread.snapshot();

                if snap.total_candidates == 0 {
                    // Designs are still being read/imported; subgraph hasn't started.
                    let ch = frames[fi % frames.len()];
                    fi += 1;
                    print!("\r{} loading designs and building indices...", ch);
                } else {
                    // Show the real progress bar once total is known.
                    let line = render_progress_bar(snap);
                    if first {
                        first = false;
                    }
                    print!("\r{}", line);
                }

                let _ = io::stdout().flush();
                thread::sleep(Duration::from_millis(100));
            }

            // Final snapshot/line
            let snap = p_for_thread.snapshot();
            if snap.total_candidates == 0 {
                println!("\r✓ finished (no candidates)");
            } else {
                let line = render_progress_bar(snap);
                println!("\r{}", line);
            }
            let _ = io::stdout().flush();
        });

        // Progress-aware dispatch (netlist queries will update progress during subgraph search).
        let count = gen_dispatch::run_count_for_type_name_with_progress(
            &args.query,
            &driver,
            &args.haystack,
            &args.module,
            &cfg,
            &progress,
        )
        .map_err(|e| {
            format!(
                "Failed to run query '{}': {}\nKnown types:\n  {}",
                args.query,
                e,
                gen_dispatch::known_query_type_names().join("\n  ")
            )
        })?;

        // Stop the renderer and wait for it to finish
        stop.store(true, Ordering::Relaxed);
        let _ = handle.join();

        println!("{}", count);
    } else {
        // No progress display: use the original path
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
    }

    // If svql_subgraph "profiling" feature is enabled, print timings
    svql_subgraph::profiling::report();

    Ok(())
}
