#![allow(dead_code)]

mod constraints;
mod embedding;
mod mapping;
mod matcher;
mod utils;

pub use embedding::{Embedding, EmbeddingSet};
pub use mapping::Assignment;
pub use matcher::{SubgraphMatcher, SubgraphMatcherCore};
pub use utils::{intersect_sets, intersect_sets_ref};
