//! Configuration for the subgraph isomorphism search.
//!
//! This module exposes a minimal, stable surface for consumers (for example,
//! the `svql_query` crate) to parameterize how matching should behave.
//!
//! The two main concepts are:
//! - match_length: whether input arity must match exactly, or whether the
//!   design can have a superset of inputs (deterministically aligned).
//! - dedupe: how to deduplicate matches after search, controlled by `DedupeMode`:
//!     - `None`: include boundary (IO) bindings when deduplicating.
//!     - `AutoMorph`: collapse permutations/automorphisms by deduping on the
//!       set of matched design gates only (ignoring boundary bindings).
//!
//! Quick examples
//!
//! Exact-length, None dedupe (original/default behavior):
//! ```ignore
//! use svql_subgraph::{Config, DedupeMode};
//! let cfg = Config::new(true, DedupeMode::None);
//! ```
//!
//! AutoMorph dedupe (collapse permutations/automorphisms) with exact-length:
//! ```ignore
//! use svql_subgraph::{Config, DedupeMode};
//! let cfg = Config::new(true, DedupeMode::AutoMorph);
//! ```
//!
//! Superset-length (allow design gates to have extra inputs) with AutoMorph dedupe:
//! ```ignore
//! use svql_subgraph::{Config, DedupeMode};
//! let cfg = Config::new(false, DedupeMode::AutoMorph);
//! ```
//!
//! Builder usage (recommended for future-proofing):
//! ```ignore
//! use svql_subgraph::{Config, DedupeMode};
//!
//! // Equivalent to Config::new(true, DedupeMode::None)
//! let cfg_default = Config::builder().exact_length().none().build();
//!
//! // Superset-length and AutoMorph dedupe
//! let cfg_superset = Config::builder()
//!     .superset_length()
//!     .auto_morph()
//!     .build();
//!
//! // Or mix and match explicitly
//! let cfg_explicit = Config::builder()
//!     .match_length(false)
//!     .dedupe(DedupeMode::AutoMorph)
//!     .build();
//! ```

#[derive(Clone, Debug)]
pub struct Config {
    /// Whether to require exact pin-count (true) or allow superset arity in the design (false).
    pub match_length: bool,
    /// How to deduplicate matches after search.
    pub dedupe: DedupeMode,
}

impl Config {
    /// Create a new configuration.
    ///
    /// - match_length:
    ///     - true  => exact-length mode (pattern inputs count must equal design).
    ///     - false => superset-length mode (design may have extra inputs).
    /// - dedupe (`DedupeMode`): how to deduplicate results after the search completes.
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
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Convenience: exact-length matching with the provided dedupe mode.
    pub fn exact_length(dedupe: DedupeMode) -> Self {
        Self::new(true, dedupe)
    }

    /// Convenience: superset-length matching with the provided dedupe mode.
    pub fn superset_length(dedupe: DedupeMode) -> Self {
        Self::new(false, dedupe)
    }
}

impl Default for Config {
    /// Default configuration mirrors the historical behavior:
    /// exact-length matching and None dedupe.
    fn default() -> Self {
        Self::new(true, DedupeMode::None)
    }
}

/// Control how matches are deduplicated.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub fn match_length(mut self, value: bool) -> Self {
        self.match_length = value;
        self
    }

    /// Convenience: request exact-length matching.
    pub fn exact_length(mut self) -> Self {
        self.match_length = true;
        self
    }

    /// Convenience: request superset-length matching.
    pub fn superset_length(mut self) -> Self {
        self.match_length = false;
        self
    }

    /// Set the dedupe mode explicitly.
    pub fn dedupe(mut self, mode: DedupeMode) -> Self {
        self.dedupe = mode;
        self
    }

    /// Convenience: request AutoMorph dedupe.
    pub fn auto_morph(mut self) -> Self {
        self.dedupe = DedupeMode::AutoMorph;
        self
    }

    /// Convenience: request None dedupe.
    pub fn none(mut self) -> Self {
        self.dedupe = DedupeMode::None;
        self
    }

    /// Finalize and construct the Config.
    pub fn build(self) -> Config {
        Config {
            match_length: self.match_length,
            dedupe: self.dedupe,
        }
    }
}
