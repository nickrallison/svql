//! Configuration for Yosys module processing.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Configuration options for processing a Yosys module.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    /// Whether to flatten the module.
    pub flatten: bool,
    /// Whether to run `opt_clean`.
    pub opt_clean: bool,
    /// Whether to run `opt`.
    pub opt: bool,
    /// Parameters to set on the module.
    pub params: HashMap<String, String>,
    /// Additional Yosys commands to run.
    pub other_steps: Vec<String>,
    /// Whether to use Verific for parsing.
    pub verific: bool,
}

impl ModuleConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_flatten(mut self, flatten: bool) -> Self {
        self.flatten = flatten;
        self
    }

    pub fn with_opt_clean(mut self, opt_clean: bool) -> Self {
        self.opt_clean = opt_clean;
        self
    }

    pub fn with_opt(mut self, opt: bool) -> Self {
        self.opt = opt;
        self
    }

    pub fn with_param(mut self, param: &str, value: &str) -> Self {
        self.params.insert(param.to_string(), value.to_string());
        self
    }

    pub fn with_step(mut self, step: &str) -> Self {
        self.other_steps.push(step.to_string());
        self
    }
    pub fn with_verific(mut self, verific: bool) -> Self {
        self.verific = verific;
        self
    }
}

impl Hash for ModuleConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.flatten.hash(state);
        self.opt_clean.hash(state);
        self.opt.hash(state);
        for (k, v) in &self.params {
            k.hash(state);
            v.hash(state);
        }
        for step in &self.other_steps {
            step.hash(state);
        }
        self.verific.hash(state);
    }
}
