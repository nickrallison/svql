//! Subgraph isomorphism matching engine and constraints.

mod assignment;
mod constraints;
mod matcher;

pub use assignment::{AssignmentSet, SingleAssignment};
pub use matcher::{SubgraphMatcher, SubgraphMatcherCore};
