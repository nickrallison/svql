//! CLI argument parsing and configuration mapping.

use clap::{Parser, ValueEnum};
use std::fmt;
use svql_common::{Config, MatchLength, ModuleConfig};
use svql_driver::DriverKey;

// Import the registration macro
use crate::register_queries;

register_queries!(QueryArg {
    Cwe1234 => svql_query_lib::security::cwe1234::Cwe1234,
    Cwe1271 => svql_query_lib::security::cwe1271::Cwe1271,
    Cwe1280 => svql_query_lib::security::cwe1280::Cwe1280,
});

/// Specification for a single design with its own sub-flags.
#[derive(Clone, Debug)]
pub struct DesignSpec {
    /// Path to the design file
    pub path: String,
    /// Module name (required)
    pub module: String,
    /// Use raw import (default: false)
    pub use_raw: bool,
    /// Match length strategy (default: subset)
    pub match_length: MatchLengthArg,
}

impl DesignSpec {
    pub fn key(&self) -> DriverKey {
        DriverKey::new(&self.path, &self.module)
    }

    pub fn to_config(&self, parallel: bool) -> Config {
        let haystack_options = ModuleConfig {
            load_raw: self.use_raw,
            ..Default::default()
        };

        Config::builder()
            .match_length(self.match_length.clone().into())
            .haystack_options(haystack_options)
            .parallel(parallel)
            .build()
    }
}

/// Custom parser for design specifications with sub-flags
fn parse_design_spec(s: &str) -> Result<DesignSpec, String> {
    // Split the string respecting quotes
    let args =
        shell_words::split(s).map_err(|e| format!("Invalid quoting in design spec: {}", e))?;

    if args.is_empty() {
        return Err("Empty design specification".to_string());
    }

    let path = args[0].clone();
    let mut module = None;
    let mut use_raw = false;
    let mut match_length = MatchLengthArg::NeedleSubsetHaystack;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--module" | "-m" => {
                i += 1;
                if i >= args.len() {
                    return Err("--module requires a value".to_string());
                }
                module = Some(args[i].clone());
            }
            "--raw" => {
                use_raw = true;
            }
            "--match-len" => {
                i += 1;
                if i >= args.len() {
                    return Err(
                        "--match-len requires a value (first, subset, or exact)".to_string()
                    );
                }
                match_length = match args[i].as_str() {
                    "first" => MatchLengthArg::First,
                    "subset" | "needle-subset-haystack" => MatchLengthArg::NeedleSubsetHaystack,
                    "exact" => MatchLengthArg::Exact,
                    other => {
                        return Err(format!(
                            "Unknown match-len: {}. Use first, subset, or exact",
                            other
                        ));
                    }
                };
            }
            _ => {
                return Err(format!(
                    "Unknown flag: {}. Allowed: --module, --raw, --match-len",
                    args[i]
                ));
            }
        }
        i += 1;
    }

    let module = module.ok_or_else(|| format!("--module is required for design: {}", path))?;

    Ok(DesignSpec {
        path,
        module,
        use_raw,
        match_length,
    })
}

/// SVQL Pattern Matcher - Search for hardware security vulnerabilities
#[derive(Parser, Debug)]
#[command(name = "svql")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Design specification(s) with their own configuration flags.
    #[arg(
        short = 'd',
        long = "design",
        value_name = "SPEC",
        value_parser = parse_design_spec,
        required = true,
        help = "Design spec: \"path.json --module NAME [--raw] [--match-len first|subset|exact]\""
    )]
    pub designs: Vec<DesignSpec>,

    /// Enable parallel execution (applies to all designs)
    #[arg(short = 'p', long, default_value_t = false)]
    pub parallel: bool,

    /// Query types to run (can be specified multiple times).
    #[arg(short = 'q', long = "query", value_enum)]
    pub queries: Vec<QueryArg>,

    /// List available queries and exit
    #[arg(long = "list-queries")]
    pub list_queries: bool,

    /// Enable profiling output (timing and memory usage)
    #[arg(long = "profile")]
    pub profile: bool,

    /// Print detailed results for all matches
    #[arg(long = "print-results")]
    pub print_results: bool,

    /// Output CSV file path for results
    #[arg(short = 'o', long = "output-csv")]
    pub output_csv: Option<String>,
}

/// Enumeration of supported match length strategies.
#[derive(Debug, Clone, ValueEnum)]
pub enum MatchLengthArg {
    First,
    NeedleSubsetHaystack,
    Exact,
}

impl From<MatchLengthArg> for MatchLength {
    fn from(arg: MatchLengthArg) -> Self {
        match arg {
            MatchLengthArg::First => Self::First,
            MatchLengthArg::NeedleSubsetHaystack => Self::NeedleSubsetHaystack,
            MatchLengthArg::Exact => Self::Exact,
        }
    }
}

impl fmt::Display for MatchLengthArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatchLengthArg::First => write!(f, "first"),
            MatchLengthArg::NeedleSubsetHaystack => write!(f, "subset"),
            MatchLengthArg::Exact => write!(f, "exact"),
        }
    }
}
