//! Configuration for the subgraph isomorphism search.

use std::{
    hash::{Hash, Hasher},
    str::FromStr,
};

use crate::ModuleConfig;

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
    /// Create a new configuration.
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
            // bind_inputs,
            needle_options,
            haystack_options,
            pattern_vars_match_design_consts,
            max_recursion_depth,
        }
    }

    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl Hash for Config {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Stub: Hash key fields (e.g., match_length, dedupe)
        self.match_length.hash(state);
        self.dedupe.hash(state);
        self.needle_options.hash(state);
        self.pattern_vars_match_design_consts.hash(state);
        self.max_recursion_depth.hash(state);
    }
}

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
    pub fn match_length(mut self, value: MatchLength) -> Self {
        self.match_length = value;
        self
    }

    pub fn dedupe(mut self, value: Dedupe) -> Self {
        self.dedupe = value;
        self
    }

    pub fn needle_options(mut self, options: ModuleConfig) -> Self {
        self.needle_options = options;
        self.needle_options = self.needle_options.with_flatten(true);
        self
    }

    pub fn needle_cmd(mut self, cmd: &str) -> Self {
        self.needle_options.other_steps.push(cmd.to_string());
        self
    }

    pub fn needle_param(mut self, param: &str, value: &str) -> Self {
        self.needle_options
            .params
            .insert(param.to_string(), value.to_string());
        self
    }

    pub fn haystack_options(mut self, options: ModuleConfig) -> Self {
        self.haystack_options = options;
        self
    }

    pub fn haystack_cmd(mut self, cmd: &str) -> Self {
        self.haystack_options.other_steps.push(cmd.to_string());
        self
    }

    pub fn haystack_param(mut self, param: &str, value: &str) -> Self {
        self.haystack_options
            .params
            .insert(param.to_string(), value.to_string());
        self
    }

    pub fn haystack_opt_clean(mut self, opt_clean: bool) -> Self {
        self.haystack_options = self.haystack_options.with_opt_clean(opt_clean);
        self
    }

    pub fn haystack_opt(mut self, opt: bool) -> Self {
        self.haystack_options = self.haystack_options.with_opt(opt);
        self
    }

    pub fn pattern_vars_match_design_consts(mut self, allow: bool) -> Self {
        self.pattern_vars_match_design_consts = allow;
        self
    }

    pub fn max_recursion_depth(mut self, depth: Option<usize>) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    pub fn build(self) -> Config {
        Config {
            match_length: self.match_length,
            dedupe: self.dedupe,
            // bind_inputs: self.bind_inputs,
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
    /// The needle must be a subgraph of the haystack.
    NeedleSubsetHaystack,
    /// The needle must exactly match the haystack.
    Exact,
}

impl MatchLength {
    pub fn first(&self) -> bool {
        matches!(self, MatchLength::First)
    }
    pub fn needle_subset_haystack(&self) -> bool {
        matches!(self, MatchLength::NeedleSubsetHaystack)
    }
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

/// Defines the deduplication strategy for matches.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Dedupe {
    /// No deduplication.
    None,
    /// Deduplicate based on inner matches.
    #[default]
    Inner,
    /// Deduplicate all matches.
    All,
}

impl Dedupe {
    pub fn none(&self) -> bool {
        matches!(self, Dedupe::None)
    }
    pub fn inner(&self) -> bool {
        matches!(self, Dedupe::Inner)
    }
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
