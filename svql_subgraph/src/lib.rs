//! Subgraph matching and graph algorithms for SVQL.
//!
//! This crate implements the core subgraph isomorphism algorithms, mapping logic,
//! and graph indexing used to match patterns against designs.

#![allow(dead_code)]
#![allow(clippy::similar_names)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unused_self)]
#![allow(clippy::cast_possible_truncation)]

mod assignment;
mod constraints;
mod matcher;
mod utils;

// #[cfg(test)]
// mod tests;

pub mod cell;
pub mod graph_index;

pub use assignment::{AssignmentSet, SingleAssignment};
pub use graph_index::GraphIndex;
pub use matcher::{SubgraphMatcher, SubgraphMatcherCore};
pub use utils::intersect_sets;
