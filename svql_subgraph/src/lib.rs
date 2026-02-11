//! Subgraph isomorphism matching engine.
//!
//! This crate implements the core pattern matching algorithm:
//! - **Subgraph Matching**: Algorithm for finding pattern instances in designs
//! - **Graph Indexing**: Efficient data structures for design traversal
//! - **Constraints**: Filtering logic (connectivity, assignment validation)
//! - **Cell Mapping**: Tracks which pattern cells map to design cells
//!
//! # Architecture
//!
//! The matcher works by:
//! 1. Building a graph index from the design for fast lookups
//! 2. Systematically mapping pattern cells to design cells
//! 3. Validating connectivity constraints between mapped cells
//! 4. Recording all valid mappings (assignments)
//!
//! Results are returned as `AssignmentSet` containing all valid pattern matches.

mod engine;
mod utils;

pub mod test_harness;

pub mod cell;

pub use engine::{AssignmentSet, SingleAssignment, SubgraphMatcher, SubgraphMatcherCore};
pub use utils::intersect_sets;
