//! Subgraph matching and graph algorithms for SVQL.
//!
//! This crate implements the core subgraph isomorphism algorithms, mapping logic,
//! and graph indexing used to match patterns against designs.

mod engine;
mod utils;

pub mod index;
pub mod test_harness;

pub mod cell;

pub use engine::{AssignmentSet, SingleAssignment, SubgraphMatcher, SubgraphMatcherCore};
pub use index::GraphIndex;
pub use utils::intersect_sets;
