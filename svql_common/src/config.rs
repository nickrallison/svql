//! Configuration for the subgraph isomorphism search.

#[derive(Clone, Debug)]
pub struct Config {
    /// Whether to require exact pin-count (true) or allow superset arity in the design (false).
    pub match_length: bool,
    /// How to deduplicate matches after search.
    pub dedupe: DedupeMode,
}

impl Config {
    /// Create a new configuration.
    #[contracts::debug_ensures(ret.match_length == match_length)]
    #[contracts::debug_ensures(ret.dedupe == dedupe)]
    pub fn new(match_length: bool, dedupe: DedupeMode) -> Self {
        Self {
            match_length,
            dedupe,
        }
    }

    /// Start building a configuration using the builder pattern.
    ///
    /// Defaults mirror historical behavior:
    /// - exact-length matching
    /// - None dedupe
    #[contracts::debug_ensures(ret.match_length)]
    #[contracts::debug_ensures(ret.dedupe == DedupeMode::None)]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Convenience: exact-length matching with the provided dedupe mode.
    #[contracts::debug_ensures(ret.match_length)]
    #[contracts::debug_ensures(ret.dedupe == dedupe)]
    pub fn exact_length(dedupe: DedupeMode) -> Self {
        Self::new(true, dedupe)
    }

    /// Convenience: superset-length matching with the provided dedupe mode.
    #[contracts::debug_ensures(!ret.match_length)]
    #[contracts::debug_ensures(ret.dedupe == dedupe)]
    pub fn superset_length(dedupe: DedupeMode) -> Self {
        Self::new(false, dedupe)
    }
}

impl Default for Config {
    /// Default configuration mirrors the historical behavior:
    /// exact-length matching and None dedupe.
    #[contracts::debug_ensures(ret.match_length)]
    #[contracts::debug_ensures(ret.dedupe == DedupeMode::None)]
    fn default() -> Self {
        Self::new(true, DedupeMode::None)
    }
}

/// Control how matches are deduplicated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DedupeMode {
    /// Include boundary IO bindings alongside gate mappings (maximally precise).
    None,
    /// Collapse matches that share the same mapped gate SET, ignoring boundary bindings.
    AutoMorph,
}

/// Builder for Config, providing a fluent API with sensible defaults.
///
/// Defaults:
/// - exact-length matching (match_length = true)
/// - None dedupe (dedupe = DedupeMode::None)
#[derive(Clone, Debug)]
pub struct ConfigBuilder {
    match_length: bool,
    dedupe: DedupeMode,
}

impl Default for ConfigBuilder {
    #[contracts::debug_ensures(ret.match_length)]
    #[contracts::debug_ensures(ret.dedupe == DedupeMode::None)]
    fn default() -> Self {
        Self {
            match_length: true,
            dedupe: DedupeMode::None,
        }
    }
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

    /// Set the dedupe mode explicitly.
    #[contracts::debug_ensures(ret.dedupe == mode)]
    pub fn dedupe(mut self, mode: DedupeMode) -> Self {
        self.dedupe = mode;
        self
    }

    /// Convenience: request AutoMorph dedupe.
    #[contracts::debug_ensures(ret.dedupe == DedupeMode::AutoMorph)]
    pub fn auto_morph(mut self) -> Self {
        self.dedupe = DedupeMode::AutoMorph;
        self
    }

    /// Convenience: request None dedupe.
    #[contracts::debug_ensures(ret.dedupe == DedupeMode::None)]
    pub fn none(mut self) -> Self {
        self.dedupe = DedupeMode::None;
        self
    }

    /// Finalize and construct the Config.
    #[contracts::debug_ensures(ret.match_length == self.match_length)]
    #[contracts::debug_ensures(ret.dedupe == self.dedupe)]
    pub fn build(self) -> Config {
        Config {
            match_length: self.match_length,
            dedupe: self.dedupe,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Config, DedupeMode};

    #[test]
    fn config_builder_table() {
        let cases = [
            (true, DedupeMode::None),
            (true, DedupeMode::AutoMorph),
            (false, DedupeMode::None),
            (false, DedupeMode::AutoMorph),
        ];

        for (match_length, dedupe) in cases {
            let cfg = Config::builder()
                .match_length(match_length)
                .dedupe(dedupe.clone())
                .build();
            assert_eq!(cfg.match_length, match_length);
            assert_eq!(cfg.dedupe, dedupe);
        }
    }

    #[test]
    fn config_builder_last_setter_wins() {
        let cfg = Config::builder()
            .superset_length()
            .exact_length()
            .auto_morph()
            .none()
            .build();

        assert_eq!(cfg.match_length, true);
        assert_eq!(cfg.dedupe, DedupeMode::None);
    }

    #[test]
    fn defaults_match_exact_none() {
        let cfg = Config::default();
        assert!(cfg.match_length);
        assert!(matches!(cfg.dedupe, DedupeMode::None));

        let b = Config::builder();
        let cfg2 = b.build();
        assert!(cfg2.match_length);
        assert!(matches!(cfg2.dedupe, DedupeMode::None));
    }
}
