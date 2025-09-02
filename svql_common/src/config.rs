//! Configuration for the subgraph isomorphism search.

use crate::{ModuleConfig, YosysModule};

#[derive(Clone, Debug, Default)]
pub struct Config {
    /// Whether to require exact pin-count (true) or allow superset arity in the design (false).
    pub match_length: bool,
    // /// How to deduplicate matches after search.
    // pub dedupe: DedupeMode,
    // /// Whether to flatten the search space (true) or keep it hierarchical (false).
    // pub flatten: bool,
    pub needle_options: ModuleConfig,
    pub haystack_options: ModuleConfig,
}

impl Config {
    /// Create a new configuration.
    pub fn new(
        match_length: bool,
        needle_options: ModuleConfig,
        haystack_options: ModuleConfig,
    ) -> Self {
        Self {
            match_length,
            needle_options,
            haystack_options,
        }
    }

    /// Start building a configuration using the builder pattern.
    ///
    /// Defaults mirror historical behavior:
    /// - exact-length matching
    /// - None dedupe
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

// /// Control how matches are deduplicated.
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum DedupeMode {
//     /// Include boundary IO bindings alongside gate mappings (maximally precise).
//     None,
//     /// Collapse matches that share the same mapped gate SET, ignoring boundary bindings.
//     AutoMorph,
// }

/// Builder for Config, providing a fluent API with sensible defaults.
///
/// Defaults:
/// - exact-length matching (match_length = true)
/// - None dedupe (dedupe = DedupeMode::None)
#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    match_length: bool,
    needle_options: ModuleConfig,
    haystack_options: ModuleConfig,
}

impl ConfigBuilder {
    /// Set whether pattern/design arity must match exactly (true) or if the design
    /// may have a superset of inputs (false).
    #[contracts::debug_ensures(ret.match_length == value)]
    pub fn match_length(mut self, value: bool) -> Self {
        self.match_length = value;
        self
    }

    /// Convenience: request exact-length matching.
    #[contracts::debug_ensures(ret.match_length)]
    pub fn exact_length(mut self) -> Self {
        self.match_length = true;
        self
    }

    /// Convenience: request superset-length matching.
    #[contracts::debug_ensures(!ret.match_length)]
    pub fn superset_length(mut self) -> Self {
        self.match_length = false;
        self
    }

    pub fn needle_options(mut self, options: ModuleConfig) -> Self {
        self.needle_options = options;
        self
    }

    pub fn haystack_options(mut self, options: ModuleConfig) -> Self {
        self.haystack_options = options;
        self
    }

    // /// Set the dedupe mode explicitly.
    // #[contracts::debug_ensures(ret.dedupe == mode)]
    // pub fn dedupe(mut self, mode: DedupeMode) -> Self {
    //     self.dedupe = mode;
    //     self
    // }

    // /// Convenience: request AutoMorph dedupe.
    // #[contracts::debug_ensures(ret.dedupe == DedupeMode::AutoMorph)]
    // pub fn auto_morph(mut self) -> Self {
    //     self.dedupe = DedupeMode::AutoMorph;
    //     self
    // }

    // /// Convenience: request None dedupe.
    // #[contracts::debug_ensures(ret.dedupe == DedupeMode::None)]
    // pub fn none(mut self) -> Self {
    //     self.dedupe = DedupeMode::None;
    //     self
    // }

    // pub fn unflatten(mut self) -> Self {
    //     self.flatten = false;
    //     self
    // }

    // pub fn flatten(mut self) -> Self {
    //     self.flatten = true;
    //     self
    // }

    pub fn build(self) -> Config {
        Config {
            match_length: self.match_length,
            needle_options: self.needle_options,
            haystack_options: self.haystack_options,
        }
    }
}
