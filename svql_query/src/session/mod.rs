//! DataFrame-based session management for query results.
//!
//! This module provides a `Session` struct that replaces the traditional
//! index lookup tables with polars DataFrames, enabling:
//! - Foreign-key style references between result tables
//! - Lazy rehydration of match results
//! - Efficient columnar storage of dehydrated matches

pub mod connection_builder;
mod error;
mod execution;
mod ref_type;
mod row;
mod schema;
mod store;
mod table;
mod variant_ref;

pub use connection_builder::ConnectionBuilder;
pub use error::QueryError;
pub use execution::{ExecInfo, ExecutionContext, ExecutionPlan, SearchFn};
pub use ref_type::Ref;
pub use row::Row;
pub use schema::{ColumnDef, ColumnEntry, ColumnKind, EntryArray, PatternSchema, PortDirection};
pub use store::Store;
pub use table::{AnyTable, Table};
pub use variant_ref::{StoreVariantExt, VariantIter, VariantPattern, VariantRef};

use polars::prelude::*;
use thiserror::Error;

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Design not loaded: {0}")]
    DesignNotLoaded(String),
    #[error("Query results not found for type: {0}")]
    ResultsNotFound(String),
    #[error("Polars error: {0}")]
    Polars(#[from] PolarsError),
    #[error("Invalid match index: {0}")]
    InvalidMatchIndex(u32),
    #[error("Rehydration error: {0}")]
    RehydrationError(String),
}
