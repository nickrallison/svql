use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleConfig {
    pub flatten: bool,
    pub opt_clean: bool,
    pub opt: bool,
    pub params: HashMap<String, String>,
    pub other_steps: Vec<String>,
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
