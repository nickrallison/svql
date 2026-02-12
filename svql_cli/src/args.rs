//! CLI argument parsing and configuration mapping.
//!
//! This module defines the command-line interface for the SVQL tool,
//! handling design paths, module selection, and execution constraints.

use clap::Parser;
use svql_common::*;
use svql_driver::DriverKey;

/// SVQL Pattern Matcher - Search for hardware security vulnerabilities
#[derive(Parser, Debug)]
#[command(name = "svql")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the design file (Verilog, RTLIL, or JSON)
    #[arg(
        short = 'f',
        long,
        default_value = "examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json"
    )]
    pub design_path: String,

    /// Name of the top-level module in the design
    #[arg(short = 'm', long, default_value = "e203_soc_top")]
    pub design_module: String,

    /// Use raw import (skip Yosys processing)
    #[arg(long, default_value_t = false)]
    pub use_raw_import: bool,

    /// Enable parallel execution
    #[arg(short = 'p', long, default_value_t = false)]
    pub parallel: bool,

    /// Set match length constraint
    #[arg(long, value_enum, default_value = "needle-subset-haystack")]
    pub match_length: MatchLengthArg,
}

impl Args {
    /// Converts parsed arguments into a query configuration.
    pub fn to_config(&self) -> Config {
        let haystack_options = ModuleConfig {
            load_raw: self.use_raw_import,
            ..Default::default()
        };

        Config::builder()
            .match_length(self.match_length.clone().into())
            .haystack_options(haystack_options)
            .parallel(self.parallel)
            .build()
    }

    /// Creates a design key from the provided path and module name.
    pub fn key(&self) -> DriverKey {
        DriverKey::new(self.design_path.as_str(), self.design_module.as_str())
    }
}

/// Enumeration of supported match length strategies.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum MatchLengthArg {
    /// Stop after the first match found
    #[value(name = "first")]
    First,
    /// Needle must be a strict subset of haystack in length
    #[value(name = "needle-subset-haystack")]
    NeedleSubsetHaystack,
    /// Needle and haystack must have identical length
    #[value(name = "exact")]
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
