//! Configuration for the subgraph isomorphism search.

use std::{
    hash::{Hash, Hasher},
    str::FromStr,
};

use crate::ModuleConfig;

/// Configuration parameters for the subgraph matching engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// The length of the match to find.
    pub match_length: MatchLength,
    /// How to deduplicate matches.
    pub dedupe: Dedupe,
    /// Configuration for the needle (pattern) module.
    pub needle_options: ModuleConfig,
    /// Configuration for the haystack (target) module.
    pub haystack_options: ModuleConfig,
    /// Whether pattern variables can match design constants.
    pub pattern_vars_match_design_consts: bool,
    /// Maximum recursion depth for the search.
    pub max_recursion_depth: Option<usize>,
}

impl Default for Config {
    /// Returns default configuration with First match length and Inner deduplication.
    fn default() -> Self {
        Self {
            match_length: MatchLength::First,
            dedupe: Dedupe::None,
            needle_options: Default::default(),
            haystack_options: Default::default(),
            pattern_vars_match_design_consts: true,
            max_recursion_depth: None,
        }
    }
}

impl Config {
    /// Creates a new configuration instance.
    pub fn new(
        match_length: MatchLength,
        dedupe: Dedupe,
        needle_options: ModuleConfig,
        haystack_options: ModuleConfig,
        pattern_vars_match_design_consts: bool,
        max_recursion_depth: Option<usize>,
    ) -> Self {
        Self {
            match_length,
            dedupe,
            needle_options,
            haystack_options,
            pattern_vars_match_design_consts,
            max_recursion_depth,
        }
    }

    /// Returns a builder for configuration.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl Hash for Config {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.match_length.hash(state);
        self.dedupe.hash(state);
        self.needle_options.hash(state);
        self.pattern_vars_match_design_consts.hash(state);
        self.max_recursion_depth.hash(state);
    }
}

/// A builder pattern implementation for the `Config` struct.
#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    match_length: MatchLength,
    dedupe: Dedupe,
    needle_options: ModuleConfig,
    haystack_options: ModuleConfig,
    pub pattern_vars_match_design_consts: bool,
    max_recursion_depth: Option<usize>,
}

impl ConfigBuilder {
    /// Sets the match length requirement.
    pub fn match_length(mut self, value: MatchLength) -> Self {
        self.match_length = value;
        self
    }

    /// Sets the deduplication strategy.
    pub fn dedupe(mut self, value: Dedupe) -> Self {
        self.dedupe = value;
        self
    }

    /// Sets the Yosys configuration for the needle and forces flattening.
    pub fn needle_options(mut self, options: ModuleConfig) -> Self {
        self.needle_options = options;
        self.needle_options = self.needle_options.with_flatten(true);
        self
    }

    /// Adds a custom Yosys command to the needle processing pipeline.
    pub fn needle_cmd(mut self, cmd: &str) -> Self {
        self.needle_options.other_steps.push(cmd.to_string());
        self
    }

    /// Sets a parameter for the needle module.
    pub fn needle_param(mut self, param: &str, value: &str) -> Self {
        self.needle_options
            .params
            .insert(param.to_string(), value.to_string());
        self
    }

    /// Sets the Yosys configuration for the haystack.
    pub fn haystack_options(mut self, options: ModuleConfig) -> Self {
        self.haystack_options = options;
        self
    }

    /// Adds a custom Yosys command to the haystack processing pipeline.
    pub fn haystack_cmd(mut self, cmd: &str) -> Self {
        self.haystack_options.other_steps.push(cmd.to_string());
        self
    }

    /// Sets a parameter for the haystack module.
    pub fn haystack_param(mut self, param: &str, value: &str) -> Self {
        self.haystack_options
            .params
            .insert(param.to_string(), value.to_string());
        self
    }

    /// Enables or disables the `opt_clean` pass for the haystack.
    pub fn haystack_opt_clean(mut self, opt_clean: bool) -> Self {
        self.haystack_options = self.haystack_options.with_opt_clean(opt_clean);
        self
    }

    /// Enables or disables the `opt` pass for the haystack.
    pub fn haystack_opt(mut self, opt: bool) -> Self {
        self.haystack_options = self.haystack_options.with_opt(opt);
        self
    }

    /// Configures whether pattern variables can match design constants.
    pub fn pattern_vars_match_design_consts(mut self, allow: bool) -> Self {
        self.pattern_vars_match_design_consts = allow;
        self
    }

    /// Sets the maximum recursion depth for the search algorithm.
    pub fn max_recursion_depth(mut self, depth: Option<usize>) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Finalizes the builder into a `Config` instance.
    pub fn build(self) -> Config {
        Config {
            match_length: self.match_length,
            dedupe: self.dedupe,
            needle_options: self.needle_options,
            haystack_options: self.haystack_options,
            pattern_vars_match_design_consts: self.pattern_vars_match_design_consts,
            max_recursion_depth: self.max_recursion_depth,
        }
    }
}

/// Defines how much of the pattern must match the target.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum MatchLength {
    /// Stop after the first match.
    #[default]
    First,
    /// The needle wire must be shorter than the haystack.
    NeedleSubsetHaystack,
    /// The needle and haystack wires must be the same length.
    Exact,
}

impl MatchLength {
    /// Returns true if the strategy is First.
    pub fn first(&self) -> bool {
        matches!(self, MatchLength::First)
    }
    /// Returns true if the strategy is NeedleSubsetHaystack.
    pub fn needle_subset_haystack(&self) -> bool {
        matches!(self, MatchLength::NeedleSubsetHaystack)
    }
    /// Returns true if the strategy is Exact.
    pub fn exact(&self) -> bool {
        matches!(self, MatchLength::Exact)
    }
}

impl FromStr for MatchLength {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "first" => Ok(MatchLength::First),
            "needle_subset_haystack" => Ok(MatchLength::NeedleSubsetHaystack),
            "exact" => Ok(MatchLength::Exact),
            _ => Err(()),
        }
    }
}

/// Defines how to handle duplicate results in the search output.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Dedupe {
    /// No deduplication is performed.
    None,
    /// Deduplicate based on the internal logic gates (ignoring I/O ports).
    #[default]
    Inner,
    /// Deduplicate based on the entire set of assigned cells (including I/O ports).
    All,
}

impl Dedupe {
    /// Returns true if the strategy is None.
    pub fn none(&self) -> bool {
        matches!(self, Dedupe::None)
    }
    /// Returns true if the strategy is Inner.
    pub fn inner(&self) -> bool {
        matches!(self, Dedupe::Inner)
    }
    /// Returns true if the strategy is All.
    pub fn all(&self) -> bool {
        matches!(self, Dedupe::All)
    }
}

impl FromStr for Dedupe {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Dedupe::None),
            "inner" => Ok(Dedupe::Inner),
            "all" => Ok(Dedupe::All),
            _ => Err(()),
        }
    }
}
