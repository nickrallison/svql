#![allow(dead_code)]

mod constraints;
mod mapping;
mod match_;
mod matcher;
mod utils;

pub use mapping::Assignment;
pub use match_::{Match, Matches};
pub use matcher::SubgraphMatcher;
pub use utils::{intersect_sets, intersect_sets_ref};
