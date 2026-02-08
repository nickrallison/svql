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
    #[arg(short = 'p', long, default_value_t = true)]
    pub parallel: bool,
    /// Set match length constraint
    #[arg(long, value_enum, default_value = "needle-subset-haystack")]
    pub match_length: MatchLengthArg,
}

impl Args {
    /// Convert command-line arguments into internal configuration
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

    pub fn key(&self) -> DriverKey {
        DriverKey::new(self.design_path.as_str(), self.design_module.as_str())
    }
}

/// Command-line argument wrapper for MatchLength
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum MatchLengthArg {
    /// First match only
    #[value(name = "first")]
    First,
    /// Needle must be a subset of haystack
    #[value(name = "needle-subset-haystack")]
    NeedleSubsetHaystack,
    /// Exact match required
    #[value(name = "exact")]
    Exact,
}

impl From<MatchLengthArg> for MatchLength {
    fn from(arg: MatchLengthArg) -> Self {
        match arg {
            MatchLengthArg::First => MatchLength::First,
            MatchLengthArg::NeedleSubsetHaystack => MatchLength::NeedleSubsetHaystack,
            MatchLengthArg::Exact => MatchLength::Exact,
        }
    }
}
