//! Configuration for the subgraph isomorphism search.

use core::{
    hash::{Hash, Hasher},
    str::FromStr,
};

use crate::ModuleConfig;
use contracts::*;

/// Configuration parameters for the subgraph matching engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// The length of the match to find.
    pub match_length: MatchLength,
    /// Configuration for the needle (pattern) module.
    pub needle_options: ModuleConfig,
    /// Configuration for the haystack (target) module.
    pub haystack_options: ModuleConfig,
    /// Whether pattern variables can match design constants.
    pub pattern_vars_match_design_consts: bool,
    /// Whether to run the search in parallel.
    pub parallel: bool,
}

impl Default for Config {
    /// Returns default configuration with First match length.
    fn default() -> Self {
        Self {
            match_length: MatchLength::First,
            needle_options: ModuleConfig::default(),
            haystack_options: ModuleConfig::default(),
            pattern_vars_match_design_consts: true,
            parallel: false,
        }
    }
}

impl Config {
    /// Creates a new configuration instance.
    #[must_use]
    pub const fn new(
        match_length: MatchLength,
        needle_options: ModuleConfig,
        haystack_options: ModuleConfig,
        pattern_vars_match_design_consts: bool,
        parallel: bool,
    ) -> Self {
        Self {
            match_length,
            needle_options,
            haystack_options,
            pattern_vars_match_design_consts,
            parallel,
        }
    }

    /// Returns a builder for configuration.
    #[must_use]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl Hash for Config {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.match_length.hash(state);
        self.needle_options.hash(state);
        self.pattern_vars_match_design_consts.hash(state);
    }
}

/// A builder pattern implementation for the `Config` struct.
#[derive(Clone, Debug)]
pub struct ConfigBuilder {
    /// Strategy for how many matches to find.
    match_length: MatchLength,
    /// Settings for pattern module ingestion.
    needle_options: ModuleConfig,
    /// Settings for target module ingestion.
    haystack_options: ModuleConfig,
    /// Whether logic variables in the pattern can match constant drivers (0/1) in the design.
    pub pattern_vars_match_design_consts: bool,
    /// Hard limit on search depth to prevent infinite recursion.
    max_recursion_depth: Option<usize>,
    /// Enable multi-threaded search.
    parallel: bool,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            match_length: MatchLength::default(),
            needle_options: ModuleConfig::default(),
            haystack_options: ModuleConfig::default(),
            pattern_vars_match_design_consts: true, // Match Config::default()
            max_recursion_depth: None,
            parallel: false,
        }
    }
}

impl ConfigBuilder {
    /// Sets the match length requirement.
    #[must_use]
    #[ensures(ret.match_length == value)]
    pub const fn match_length(mut self, value: MatchLength) -> Self {
        self.match_length = value;
        self
    }

    /// Sets the Yosys configuration for the needle and forces flattening.
    #[must_use]
    #[ensures(ret.needle_options.flatten)]
    pub fn needle_options(mut self, options: ModuleConfig) -> Self {
        self.needle_options = options;
        self.needle_options = self.needle_options.with_flatten(true);
        self
    }

    /// Adds a custom Yosys command to the needle processing pipeline.
    #[must_use]
    pub fn needle_cmd(mut self, cmd: &str) -> Self {
        self.needle_options.other_steps.push(cmd.to_owned());
        self
    }

    /// Sets a parameter for the needle module.
    #[must_use]
    pub fn needle_param(mut self, param: &str, value: &str) -> Self {
        self.needle_options
            .params
            .insert(param.to_owned(), value.to_owned());
        self
    }

    /// Sets a parameter for the needle module. Able to be used in a const context
    #[must_use]
    pub const fn needle_const_params(
        mut self,
        params: &'static [(&'static str, &'static str)],
    ) -> Self {
        self.needle_options.const_params = params;
        self
    }

    /// Sets the Yosys configuration for the haystack.
    #[must_use]
    pub fn haystack_options(mut self, options: ModuleConfig) -> Self {
        self.haystack_options = options;
        self
    }

    /// Adds a custom Yosys command to the haystack processing pipeline.
    #[must_use]
    pub fn haystack_cmd(mut self, cmd: &str) -> Self {
        self.haystack_options.other_steps.push(cmd.to_owned());
        self
    }

    /// Sets a parameter for the haystack module.
    #[must_use]
    pub fn haystack_param(mut self, param: &str, value: &str) -> Self {
        self.haystack_options
            .params
            .insert(param.to_owned(), value.to_owned());
        self
    }

    /// Sets a parameter for the haystack module. Able to be used in a const context
    #[must_use]
    pub const fn haystack_const_params(
        mut self,
        params: &'static [(&'static str, &'static str)],
    ) -> Self {
        self.haystack_options.const_params = params;
        self
    }

    /// Enables or disables the `opt_clean` pass for the haystack.
    #[must_use]
    pub fn haystack_opt_clean(mut self, opt_clean: bool) -> Self {
        self.haystack_options = self.haystack_options.with_opt_clean(opt_clean);
        self
    }

    /// Enables or disables the `opt` pass for the haystack.
    #[must_use]
    pub fn haystack_opt(mut self, opt: bool) -> Self {
        self.haystack_options = self.haystack_options.with_opt(opt);
        self
    }

    /// Configures whether pattern variables can match design constants.
    #[must_use]
    #[ensures(ret.pattern_vars_match_design_consts == allow)]
    pub const fn pattern_vars_match_design_consts(mut self, allow: bool) -> Self {
        self.pattern_vars_match_design_consts = allow;
        self
    }

    /// Sets the maximum recursion depth for the search algorithm.
    #[must_use]
    #[ensures(ret.max_recursion_depth == depth)]
    pub const fn max_recursion_depth(mut self, depth: Option<usize>) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Sets whether to run the search in parallel.
    #[must_use]
    #[ensures(ret.parallel == parallel)]
    pub const fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Finalizes the builder into a `Config` instance.
    #[must_use]
    pub fn build(self) -> Config {
        Config {
            match_length: self.match_length,
            needle_options: self.needle_options,
            haystack_options: self.haystack_options,
            pattern_vars_match_design_consts: self.pattern_vars_match_design_consts,
            parallel: self.parallel,
        }
    }
}

/// Defines matching strategies for pattern searches.
///
/// Different strategies control how much of the pattern must match the target design.
#[derive(Clone, Copy, Debug, Default, Hash)]
#[derive_const(PartialEq, Eq)]
pub enum MatchLength {
    /// Stop after the first valid match is found.
    /// Fastest execution but only finds one result.
    #[default]
    First,
    /// The pattern (needle) must be a strict subset of the target (haystack) in length.
    /// Useful for finding patterns embedded within larger structures.
    NeedleSubsetHaystack,
    /// The pattern and target must have identical structure length.
    /// Requires exact matches with no additional nodes.
    Exact,
}

impl MatchLength {
    /// Returns true if the strategy is `First`.
    #[must_use]
    pub const fn first(&self) -> bool {
        matches!(self, Self::First)
    }

    /// Returns true if the strategy is `NeedleSubsetHaystack`.
    #[must_use]
    pub const fn needle_subset_haystack(&self) -> bool {
        matches!(self, Self::NeedleSubsetHaystack)
    }

    /// Returns true if the strategy is `Exact`.
    #[must_use]
    pub const fn exact(&self) -> bool {
        matches!(self, Self::Exact)
    }
}

impl FromStr for MatchLength {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "first" => Ok(Self::First),
            "needle_subset_haystack" => Ok(Self::NeedleSubsetHaystack),
            "exact" => Ok(Self::Exact),
            _ => Err(()),
        }
    }
}
