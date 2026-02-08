//! Query result storage and execution context.
//!
//! Provides infrastructure for:
//! - **Schema**: Type-safe descriptions of pattern component structure
//! - **Storage**: Columnar tables for efficient result storage and retrieval
//! - **Execution**: Query planning and execution context management
//! - **Connection**: Builder utilities for establishing pattern component relationships
//!
//! Results are stored in a `Store` which acts as a central registry of all
//! result tables, allowing patterns to access dependencies during search.

pub mod execution;
pub mod storage;

pub mod connection_builder;
mod error;
pub mod schema;

pub use connection_builder::ConnectionBuilder;
pub use error::QueryError;
pub use execution::{ExecInfo, ExecutionContext, ExecutionPlan, SearchFn};
pub use schema::{
    Alias, ColumnDef, ColumnEntry, ColumnKind, EntryArray, PatternSchema, Port, PortMap, Submodule,
};
pub use storage::{AnyTable, ColumnStore, Ref, Row, Store, Table};

use thiserror::Error;

/// Errors that can occur during session management.
#[derive(Debug, Error)]
pub enum SessionError {
    /// A required design was not loaded by the driver.
    #[error("Design not loaded: {0}")]
    DesignNotLoaded(String),
    /// Query results for a pattern type were not found in the store.
    #[error("Query results not found for type: {0}")]
    ResultsNotFound(String),
    /// Invalid or out-of-range index into match results.
    #[error("Invalid match index: {0}")]
    InvalidMatchIndex(u32),
    /// Failed to reconstruct a pattern from stored results.
    #[error("Rehydration error: {0}")]
    RehydrationError(String),
}
