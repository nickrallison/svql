//! Configuration for the subgraph isomorphism search.

use crate::{ModuleConfig, YosysModule};

#[derive(Clone, Debug)]
pub struct Config {
    pub match_length: bool,
    pub dedupe: bool,
    pub bind_inputs: bool,
    pub needle_options: ModuleConfig,
    pub haystack_options: ModuleConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            match_length: false,
            dedupe: false,
            bind_inputs: true,
            needle_options: Default::default(),
            haystack_options: Default::default(),
        }
    }
}

impl Config {
    /// Create a new configuration.
    pub fn new(
        match_length: bool,
        dedupe: bool,
        bind_inputs: bool,
        needle_options: ModuleConfig,
        haystack_options: ModuleConfig,
    ) -> Self {
        Self {
            match_length,
            dedupe,
            bind_inputs,
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
    match_length: bool,
    dedupe: bool,
    bind_inputs: bool,
    needle_options: ModuleConfig,
    haystack_options: ModuleConfig,
}

impl ConfigBuilder {
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

    pub fn bind_inputs(mut self, value: bool) -> Self {
        self.bind_inputs = value;
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
            match_length: self.match_length,
            dedupe: self.dedupe,
            bind_inputs: self.bind_inputs,
            needle_options: self.needle_options,
            haystack_options: self.haystack_options,
        }
    }
}
