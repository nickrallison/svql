//! Session management for query results.
//!
//! This module provides a `Session` struct that uses simple columnar storage
//! for pattern results, enabling:
//! - Foreign-key style references between result tables
//! - Lazy rehydration of match results
//! - Efficient columnar storage of dehydrated matches

mod columnar;
pub mod connection_builder;
pub mod connectivity_index;
mod error;
mod execution;
pub mod join_planner;
mod ref_type;
mod row;
mod schema;
mod store;
mod table;
// mod variant_ref;

pub use columnar::ColumnStore;
pub use connection_builder::ConnectionBuilder;
pub use error::QueryError;
pub use execution::{ExecInfo, ExecutionContext, ExecutionPlan, SearchFn};
pub use ref_type::Ref;
pub use row::Row;
pub use schema::{
    Alias, ColumnDef, ColumnEntry, ColumnKind, EntryArray, PatternSchema, Port, PortDirection,
    PortMap, Submodule,
};
pub use store::Store;
pub use table::{AnyTable, Table};
// pub use variant_ref::{StoreVariantExt, VariantIter, VariantPattern, VariantRef};

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
