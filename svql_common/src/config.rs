//! Configuration for the subgraph isomorphism search.

use std::str::FromStr;

use crate::ModuleConfig;

#[derive(Clone, Debug)]
pub struct Config {
    pub match_length: MatchLength,
    pub dedupe: Dedupe,
    // pub bind_inputs: bool,
    pub needle_options: ModuleConfig,
    pub haystack_options: ModuleConfig,
    pub pattern_vars_match_design_consts: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            match_length: MatchLength::First,
            dedupe: Dedupe::None,
            // bind_inputs: true,
            needle_options: Default::default(),
            haystack_options: Default::default(),
            pattern_vars_match_design_consts: true,
        }
    }
}

impl Config {
    /// Create a new configuration.
    pub fn new(
        match_length: MatchLength,
        dedupe: Dedupe,
        // bind_inputs: bool,
        needle_options: ModuleConfig,
        haystack_options: ModuleConfig,
        pattern_vars_match_design_consts: bool,
    ) -> Self {
        Self {
            match_length,
            dedupe,
            // bind_inputs,
            needle_options,
            haystack_options,
            pattern_vars_match_design_consts,
        }
    }

    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    match_length: MatchLength,
    dedupe: Dedupe,
    // bind_inputs: bool,
    needle_options: ModuleConfig,
    haystack_options: ModuleConfig,
    pub pattern_vars_match_design_consts: bool,
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

    // pub fn bind_inputs(mut self, value: bool) -> Self {
    //     self.bind_inputs = value;
    //     self
    // }

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

    // pub fn needle_flatten(mut self, flatten: bool) -> Self {
    //     self.needle_options.flatten = flatten;
    //     self
    // }

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

    // pub fn haystack_flatten(mut self, flatten: bool) -> Self {
    //     self.haystack_options.flatten = flatten;
    //     self
    // }

    pub fn pattern_vars_match_design_consts(mut self, allow: bool) -> Self {
        self.pattern_vars_match_design_consts = allow;
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
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum MatchLength {
    #[default]
    First,
    NeedleSubsetHaystack,
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Dedupe {
    None,
    #[default]
    Inner,
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

// from str
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
