//! Configuration for the subgraph isomorphism search.

use crate::{ModuleConfig, YosysModule};

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub parallel: bool,
    pub match_length: bool,
    pub dedupe: bool,
    pub needle_options: ModuleConfig,
    pub haystack_options: ModuleConfig,
}

impl Config {
    /// Create a new configuration.
    pub fn new(
        parallel: bool,
        match_length: bool,
        dedupe: bool,
        needle_options: ModuleConfig,
        haystack_options: ModuleConfig,
    ) -> Self {
        Self {
            parallel,
            match_length,
            dedupe,
            needle_options,
            haystack_options,
        }
    }

    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    parallel: bool,
    match_length: bool,
    dedupe: bool,
    needle_options: ModuleConfig,
    haystack_options: ModuleConfig,
}

impl ConfigBuilder {
    pub fn parallel(mut self, value: bool) -> Self {
        self.parallel = value;
        self
    }
    pub fn match_length(mut self, value: bool) -> Self {
        self.match_length = value;
        self
    }

    pub fn exact_length(mut self) -> Self {
        self.match_length = true;
        self
    }

    pub fn superset_length(mut self) -> Self {
        self.match_length = false;
        self
    }
    pub fn dedupe(mut self, value: bool) -> Self {
        self.dedupe = value;
        self
    }

    pub fn needle_options(mut self, options: ModuleConfig) -> Self {
        self.needle_options = options;
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

    pub fn needle_flatten(mut self, flatten: bool) -> Self {
        self.needle_options.flatten = flatten;
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

    pub fn haystack_flatten(mut self, flatten: bool) -> Self {
        self.haystack_options.flatten = flatten;
        self
    }

    pub fn build(self) -> Config {
        Config {
            parallel: self.parallel,
            match_length: self.match_length,
            dedupe: self.dedupe,
            needle_options: self.needle_options,
            haystack_options: self.haystack_options,
        }
    }
}
