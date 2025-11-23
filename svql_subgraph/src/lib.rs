//! Subgraph matching and graph algorithms for SVQL.
//!
//! This crate implements the core subgraph isomorphism algorithms, embedding logic,
//! and graph indexing used to match patterns against designs.

#![allow(dead_code)]

mod constraints;
mod embedding;
mod mapping;
mod matcher;
mod utils;

pub mod cell;
pub mod graph_index;

pub use embedding::{Embedding, EmbeddingSet};
pub use matcher::{SubgraphMatcher, SubgraphMatcherCore};
pub use utils::{intersect_sets, intersect_sets_ref};

// Re-export commonly used types for convenience
pub use graph_index::GraphIndex;
pub use mapping::Assignment;
