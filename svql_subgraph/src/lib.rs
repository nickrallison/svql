//! Subgraph matching and graph algorithms for SVQL.
//!
//! This crate implements the core subgraph isomorphism algorithms, mapping logic,
//! and graph indexing used to match patterns against designs.

#![allow(dead_code)]

mod assignment;
mod constraints;
mod matcher;
mod utils;

pub mod cell;
pub mod graph_index;

pub use assignment::{AssignmentSet, SingleAssignment};
pub use graph_index::GraphIndex;
pub use matcher::{SubgraphMatcher, SubgraphMatcherCore};
pub use utils::{intersect_sets, intersect_sets_ref};
