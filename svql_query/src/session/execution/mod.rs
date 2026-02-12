//! Query execution planning and dispatch.

mod connectivity_index;
/// Cost-based planner for joining submodule results.
pub mod join_planner;
mod plan;
mod slot;

pub use connectivity_index::*;
pub use join_planner::*;
pub use plan::{ExecInfo, ExecutionContext, ExecutionPlan, SearchFn};
pub use slot::TableSlot;
