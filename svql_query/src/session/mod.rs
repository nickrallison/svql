//! DataFrame-based session management for query results.
//!
//! This module provides a `Session` struct that replaces the traditional
//! index lookup tables with polars DataFrames, enabling:
//! - Foreign-key style references between result tables
//! - Lazy rehydration of match results
//! - Efficient columnar storage of dehydrated matches
//!
//! ## New API (Phase 1-3)
//!
//! The following types are part of the new DataFrame-based API:
//! - [`CellId`] - 64-bit cell identifier with multi-design support
//! - [`Ref<T>`](ref_type::Ref) - Typed reference to a row in another pattern's table
//! - [`Row<T>`] - Owned snapshot of a single row
//! - [`Table<T>`] - Typed DataFrame wrapper
//! - [`Store`] - Central storage for all pattern result tables
//! - [`ColumnDef`], [`ColumnKind`] - Schema definitions
//! - [`QueryError`] - Error types
//! - [`PatternRegistry`] - Type registration for DAG construction
//! - [`ExecutionPlan`], [`ExecutionContext`] - Parallel execution

// --- New modules (Phase 1-3) ---
mod cell_id;
mod column;
mod error;
mod execution;
mod ref_type;
mod registry;
mod row;
mod store;
mod table;
mod tree;
mod variant_ref;

pub use cell_id::CellId;
pub use column::{ColumnDef, ColumnKind};
pub use error::QueryError;
pub use execution::{ExecutionContext, ExecutionNode, ExecutionPlan, SearchFn};
pub use ref_type::Ref;
pub use registry::{PatternEntry, PatternRegistry, SearchRegistry};
pub use row::Row;
pub use store::Store;
pub use table::{AnyTable, Table, TableBuilder};
pub use tree::{
    TempRef, TreePostOrderIter, TreePreOrderIter, TreeRowBuilder, TreeRowExt, TreeTableBuilder,
};
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
