//! Configuration for the subgraph isomorphism search.
//!
//! This module exposes a minimal, stable surface for consumers (for example,
//! the `svql_query` crate) to parameterize how matching should behave.
//!
//! The two main concepts are:
//! - match_length: whether input arity must match exactly, or whether the
//!   design can have a superset of inputs (deterministically aligned).
//! - dedupe: how to deduplicate matches after search (by full boundary
//!   bindings, or by the set of matched design gates only).
//!
//! Quick examples
//!
//! Exact-length, full dedupe (original/default behavior):
//! ```ignore
//! use svql_subgraph::{Config, DedupeMode};
//! let cfg = Config::new(true, DedupeMode::Full);
//! ```
//!
//! Gates-only dedupe (collapse permutations/automorphisms) with exact-length:
//! ```ignore
//! use svql_subgraph::{Config, DedupeMode};
//! let cfg = Config::new(true, DedupeMode::GatesOnly);
//! ```
//!
//! Superset-length (allow design gates to have extra inputs) with gates-only dedupe:
//! ```ignore
//! use svql_subgraph::{Config, DedupeMode};
//! let cfg = Config::new(false, DedupeMode::GatesOnly);
//! ```

/// Global search configuration.
///
/// - match_length:
///     - true  => enforce that the number of inputs (pins) on matched gates
///       is exactly equal between the pattern and the design.
///     - false => allow the design gate to have a superset of inputs. In this
///       mode, the algorithm compares the first N inputs after a deterministic
///       alignment rule (commutative gates are normalized), where N is the
///       number of pattern inputs. This retains determinism while avoiding
///       combinatorial explosion from trying all subsets.
/// - dedupe:
///     - Controls how matches are deduplicated after search:
///         - Full: include boundary (IO) bindings in the dedupe signature
///                 (original behavior).
///         - GatesOnly: ignore IO/boundary bindings and collapse matches that
///                 map to the same SET of design gates (i.e., collapse
///                 permutations/automorphisms).
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
    /// - dedupe: how to deduplicate results after the search completes.
    ///
    /// Examples:
    /// - Original behavior:
    ///   `Config::new(true, DedupeMode::Full)`
    /// - Collapse automorphisms (permutations of pattern mapping):
    ///   `Config::new(true, DedupeMode::GatesOnly)`
    /// - Allow extra inputs in design and dedupe by gates only:
    ///   `Config::new(false, DedupeMode::GatesOnly)`
    pub fn new(match_length: bool, dedupe: DedupeMode) -> Self {
        Self {
            match_length,
            dedupe,
        }
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
    /// exact-length matching and Full dedupe.
    fn default() -> Self {
        Self::new(true, DedupeMode::Full)
    }
}

/// Control how matches are deduplicated.
///
/// - Full:
///     Include boundary IO bindings alongside gate mappings. Two matches are
///     considered distinct if they differ in any gate mapping OR any boundary
///     binding. This is the most precise (and original) behavior.
/// - GatesOnly:
///     Ignore boundary IO bindings entirely. Two matches are considered the
///     same if they map to the same SET of design gates, regardless of which
///     pattern gates map to which design gates (collapses permutations and
///     automorphisms).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DedupeMode {
    /// Include boundary bindings in the dedupe signature.
    Full,
    /// Collapse matches that share the same mapped gate Set, ignoring boundary bindings.
    GatesOnly,
}
