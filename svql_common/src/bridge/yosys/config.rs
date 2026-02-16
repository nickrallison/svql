//! Configuration for Yosys module processing.

use crate::*;
use contracts::*;
use core::hash::{Hash, Hasher};

/// Configuration options for processing a Yosys module.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ModuleConfig {
    /// Whether to flatten the module.
    pub flatten: bool,
    /// Whether to run `opt_clean`.
    pub opt_clean: bool,
    /// Whether to run `opt`.
    pub opt: bool,
    /// Parameters to set on the module.
    pub params: HashMap<String, String>,
    /// Parameters to be set on the module compatable with const functions
    pub const_params: &'static [(&'static str, &'static str)],
    /// Additional Yosys commands to run.
    pub other_steps: Vec<String>,
    /// Whether to use Verific for parsing.
    pub verific: bool,
    /// Whether to load the raw unprocessed design, must be in json format.
    pub load_raw: bool,
}

impl ModuleConfig {
    /// Creates a new default configuration for Yosys module processing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to flatten the module hierarchy.
    #[must_use]
    #[ensures(ret.flatten == flatten)]
    pub fn with_flatten(mut self, flatten: bool) -> Self {
        self.flatten = flatten;
        self
    }

    /// Sets whether to run the `opt_clean` pass.
    #[must_use]
    #[ensures(ret.opt_clean == opt_clean)]
    pub fn with_opt_clean(mut self, opt_clean: bool) -> Self {
        self.opt_clean = opt_clean;
        self
    }

    /// Sets whether to run the `opt` pass.
    #[must_use]
    #[ensures(ret.opt == opt)]
    pub fn with_opt(mut self, opt: bool) -> Self {
        self.opt = opt;
        self
    }

    /// Adds a parameter override.
    #[must_use]
    #[ensures(ret.params.get(param) == Some(&value.to_string()))]
    pub fn with_param(mut self, param: &str, value: &str) -> Self {
        self.params.insert(param.to_owned(), value.to_owned());
        self
    }

    /// Adds a custom Yosys step command.
    #[must_use]
    #[ensures(ret.other_steps.last() == Some(&step.to_string()))]
    pub fn with_step(mut self, step: &str) -> Self {
        self.other_steps.push(step.to_owned());
        self
    }

    /// Sets whether to use Verific for parsing.
    #[must_use]
    #[ensures(ret.verific == verific)]
    pub fn with_verific(mut self, verific: bool) -> Self {
        self.verific = verific;
        self
    }

    /// Sets whether to skip Yosys processing and load the raw JSON.
    #[must_use]
    #[ensures(ret.load_raw == load_raw)]
    pub fn with_load_raw(mut self, load_raw: bool) -> Self {
        self.load_raw = load_raw;
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
