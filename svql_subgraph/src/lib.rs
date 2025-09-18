#![allow(dead_code)]

mod constraints;
mod embedding;
mod mapping;
mod matcher;
mod utils;

pub use embedding::{Embedding, EmbeddingSet};
pub use matcher::{SubgraphMatcher, SubgraphMatcherCore};
pub use utils::{intersect_sets, intersect_sets_ref};

// Re-export commonly used types for convenience
pub use mapping::Assignment;
