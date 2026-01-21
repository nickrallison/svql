//! Error types for query operations.
//!
//! Most errors should occur during the **preparation phase** (plan construction,
//! design loading, needle validation) rather than during parallel search.

use std::any::TypeId;

use polars::prelude::PolarsError;
use thiserror::Error;

/// Errors that can occur during query preparation or execution.
///
/// Design principle: Most errors should occur during the **preparation phase**
/// (plan construction, design loading, needle validation) rather than during
/// the parallel search phase. This keeps the hot path simple and avoids
/// complex error handling in parallel code.
#[derive(Debug, Error)]
pub enum QueryError {
    // === Preparation phase errors (before search) ===
    /// Failed to load the design file.
    #[error("Failed to load design: {0}")]
    DesignLoad(String),

    /// Failed to load the needle (pattern) design.
    #[error("Failed to load needle design: {0}")]
    NeedleLoad(String),

    /// Schema mismatch between expected and actual column types.
    #[error("Schema mismatch: expected {expected}, got {actual}")]
    SchemaMismatch { expected: String, actual: String },

    /// Pattern type was not registered before execution.
    #[error("Pattern type not registered: {0:?}")]
    MissingRegistration(TypeId),

    // === Execution phase errors (rare, indicate bugs or resource issues) ===
    /// DataFrame/Polars operation failed.
    #[error("DataFrame operation failed: {0}")]
    DataFrame(#[from] PolarsError),

    /// Missing dependency table during search (indicates DAG ordering bug).
    #[error("Internal: missing dependency table '{0}' (DAG ordering bug)")]
    MissingDependency(String),

    /// Subgraph matching failed.
    #[error("Subgraph matching failed: {0}")]
    SubgraphMatch(String),
}

impl QueryError {
    /// Create a design load error.
    pub fn design_load(msg: impl Into<String>) -> Self {
        Self::DesignLoad(msg.into())
    }

    /// Create a needle load error.
    pub fn needle_load(msg: impl Into<String>) -> Self {
        Self::NeedleLoad(msg.into())
    }

    /// Create a missing dependency error.
    pub fn missing_dep(name: impl Into<String>) -> Self {
        Self::MissingDependency(name.into())
    }

    /// Create a subgraph match error.
    pub fn subgraph_match(msg: impl Into<String>) -> Self {
        Self::SubgraphMatch(msg.into())
    }
}
