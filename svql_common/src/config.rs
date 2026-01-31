//! Configuration for the subgraph isomorphism search.

use core::{
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
    /// Whether to run the search in parallel.
    pub parallel: bool,
}

impl Default for Config {
    /// Returns default configuration with First match length and Inner deduplication.
    fn default() -> Self {
        Self {
            match_length: MatchLength::First,
            dedupe: Dedupe::None,
            needle_options: ModuleConfig::default(),
            haystack_options: ModuleConfig::default(),
            pattern_vars_match_design_consts: true,
            max_recursion_depth: None,
            parallel: false,
        }
    }
}

impl Config {
    /// Creates a new configuration instance.
    #[must_use]
    pub const fn new(
        match_length: MatchLength,
        dedupe: Dedupe,
        needle_options: ModuleConfig,
        haystack_options: ModuleConfig,
        pattern_vars_match_design_consts: bool,
        max_recursion_depth: Option<usize>,
        parallel: bool,
    ) -> Self {
        Self {
            match_length,
            dedupe,
            needle_options,
            haystack_options,
            pattern_vars_match_design_consts,
            max_recursion_depth,
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
    parallel: bool,
}

impl ConfigBuilder {
    /// Sets the match length requirement.
    #[must_use]
    pub const fn match_length(mut self, value: MatchLength) -> Self {
        self.match_length = value;
        self
    }

    /// Sets the deduplication strategy.
    #[must_use]
    pub const fn dedupe(mut self, value: Dedupe) -> Self {
        self.dedupe = value;
        self
    }

    /// Sets the Yosys configuration for the needle and forces flattening.
    #[must_use]
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
    pub const fn pattern_vars_match_design_consts(mut self, allow: bool) -> Self {
        self.pattern_vars_match_design_consts = allow;
        self
    }

    /// Sets the maximum recursion depth for the search algorithm.
    #[must_use]
    pub const fn max_recursion_depth(mut self, depth: Option<usize>) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Sets whether to run the search in parallel.
    #[must_use]
    pub const fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Finalizes the builder into a `Config` instance.
    #[must_use]
    pub fn build(self) -> Config {
        Config {
            match_length: self.match_length,
            dedupe: self.dedupe,
            needle_options: self.needle_options,
            haystack_options: self.haystack_options,
            pattern_vars_match_design_consts: self.pattern_vars_match_design_consts,
            max_recursion_depth: self.max_recursion_depth,
            parallel: self.parallel,
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
    #[must_use]
    pub const fn first(&self) -> bool {
        matches!(self, Self::First)
    }
    /// Returns true if the strategy is `NeedleSubsetHaystack`.
    #[must_use]
    pub const fn needle_subset_haystack(&self) -> bool {
        matches!(self, Self::NeedleSubsetHaystack)
    }
    /// Returns true if the strategy is Exact.
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
    #[must_use]
    pub const fn none(&self) -> bool {
        matches!(self, Self::None)
    }
    /// Returns true if the strategy is Inner.
    #[must_use]
    pub const fn inner(&self) -> bool {
        matches!(self, Self::Inner)
    }
    /// Returns true if the strategy is All.
    #[must_use]
    pub const fn all(&self) -> bool {
        matches!(self, Self::All)
    }
}

impl FromStr for Dedupe {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "inner" => Ok(Self::Inner),
            "all" => Ok(Self::All),
            _ => Err(()),
        }
    }
}
