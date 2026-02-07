//! Session management for query results.
//!
//! This module provides a `Session` struct that uses simple columnar storage
//! for pattern results, enabling:
//! - Foreign-key style references between result tables
//! - Lazy rehydration of match results
//! - Efficient columnar storage of dehydrated matches

pub mod execution;
pub mod storage;

pub mod connection_builder;
pub mod schema;
mod error;

pub use connection_builder::ConnectionBuilder;
pub use error::QueryError;
pub use execution::{ExecInfo, ExecutionContext, ExecutionPlan, SearchFn};
pub use schema::{
    Alias, ColumnDef, ColumnEntry, ColumnKind, EntryArray, PatternSchema, Port, PortMap, Submodule,
};
pub use storage::{ColumnStore, Ref, Row, Store, Table, AnyTable};

use thiserror::Error;

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Design not loaded: {0}")]
    DesignNotLoaded(String),
    #[error("Query results not found for type: {0}")]
    ResultsNotFound(String),
    #[error("Invalid match index: {0}")]
    InvalidMatchIndex(u32),
    #[error("Rehydration error: {0}")]
    RehydrationError(String),
}
