//! Lazy rehydration of dehydrated match results.
//!
//! Provides traits and utilities for converting DataFrame-stored results
//! back into full `Match` state objects on demand.
//!
//! # Deprecation Notice
//!
//! This module is deprecated and will be removed in a future version.
//! Use the new DataFrame-based API with `Row<T>` and `Table<T>` instead:
//!
//! ```ignore
//! // Instead of:
//! let ctx = RehydrateContext::new(design_frame, results, design);
//! let match_obj = T::rehydrate(&row, &ctx)?;
//!
//! // Use:
//! let table: Table<T::Match> = store.table::<T>();
//! for row in table.iter() {
//!     // row provides direct access to match fields
//! }
//! ```

use prjunnamed_netlist::Design;

use super::{DesignFrame, MatchRow, ResultStore, SessionError};
use crate::prelude::*;
use crate::subgraph::cell::CellInfo;

/// Context for rehydrating dehydrated matches.
///
/// **Deprecated:** Use `Row<T>` and `Table<T>` for direct DataFrame access instead.
#[deprecated(since = "0.2.0", note = "Use Row<T> and Table<T> instead")]
#[allow(deprecated)]
pub struct RehydrateContext<'a> {
    design_frame: &'a DesignFrame,
    results: &'a ResultStore,
    #[allow(dead_code)]
    design: &'a Design,
}

#[allow(deprecated)]
impl<'a> RehydrateContext<'a> {
    /// Creates a new rehydration context.
    pub fn new(
        design_frame: &'a DesignFrame,
        results: &'a ResultStore,
        design: &'a Design,
    ) -> Self {
        Self {
            design_frame,
            results,
            design,
        }
    }

    /// Returns the design frame.
    pub fn design_frame(&self) -> &DesignFrame {
        self.design_frame
    }

    /// Returns the result store.
    pub fn results(&self) -> &ResultStore {
        self.results
    }

    /// Rehydrates a cell ID to CellInfo.
    pub fn rehydrate_cell(&self, cell_id: u32) -> Option<CellInfo> {
        self.design_frame
            .get_cell(cell_id)
            .map(|row| row.to_cell_info())
    }

    /// Rehydrates a wire from a cell ID.
    pub fn rehydrate_wire(&self, path: Instance, cell_id: Option<u32>) -> Wire<Match> {
        let cell_info = cell_id.and_then(|id| self.rehydrate_cell(id));
        Wire::new(path, cell_info)
    }

    /// Gets a match row from the result store by type name and index.
    pub fn get_match_row(&self, type_name: &str, match_idx: u32) -> Option<MatchRow> {
        self.results.get_by_name(type_name)?.get(match_idx)
    }

    /// Validates that a connection exists between two cells.
    pub fn validate_connection(&self, from_cell_id: u32, to_cell_id: u32) -> bool {
        self.design_frame.is_connected(from_cell_id, to_cell_id)
    }
}

/// Trait for types that can be rehydrated from DataFrame storage.
///
/// This is the counterpart to the `Dehydrate` trait, enabling
/// conversion from dehydrated (DataFrame row) format back to full Match objects.
///
/// **Deprecated:** Use `Row<T>` for direct field access instead.
#[deprecated(since = "0.2.0", note = "Use Row<T> for direct field access")]
pub trait Rehydrate: Sized {
    /// The type name used for lookup in the result store.
    const TYPE_NAME: &'static str;

    /// Rehydrates a match from a DataFrame row.
    fn rehydrate(row: &MatchRow, ctx: &RehydrateContext<'_>) -> Result<Self, SessionError>;

    /// Rehydrates a match by index from the result store.
    #[allow(deprecated)]
    fn rehydrate_by_index(
        match_idx: u32,
        ctx: &RehydrateContext<'_>,
    ) -> Result<Self, SessionError> {
        let row = ctx
            .get_match_row(Self::TYPE_NAME, match_idx)
            .ok_or_else(|| SessionError::InvalidMatchIndex(match_idx))?;

        Self::rehydrate(&row, ctx)
    }
}

/// Iterator that lazily rehydrates matches.
///
/// **Deprecated:** Use `Table<T>::iter()` instead.
#[deprecated(since = "0.2.0", note = "Use Table<T>::iter() instead")]
#[allow(deprecated)]
pub struct RehydrateIter<'a, T: Rehydrate> {
    ctx: &'a RehydrateContext<'a>,
    current_idx: u32,
    max_idx: u32,
    _marker: std::marker::PhantomData<T>,
}

#[allow(deprecated)]
impl<'a, T: Rehydrate> RehydrateIter<'a, T> {
    /// Creates a new rehydrating iterator.
    pub fn new(ctx: &'a RehydrateContext<'a>) -> Self {
        let max_idx = ctx
            .results()
            .get_by_name(T::TYPE_NAME)
            .map(|r| r.len() as u32)
            .unwrap_or(0);

        Self {
            ctx,
            current_idx: 0,
            max_idx,
            _marker: std::marker::PhantomData,
        }
    }
}

#[allow(deprecated)]
impl<T: Rehydrate> Iterator for RehydrateIter<'_, T> {
    type Item = Result<T, SessionError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.max_idx {
            return None;
        }

        let row = self.ctx.get_match_row(T::TYPE_NAME, self.current_idx)?;
        self.current_idx += 1;
        Some(T::rehydrate(&row, self.ctx))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.max_idx - self.current_idx) as usize;
        (remaining, Some(remaining))
    }
}

#[allow(deprecated)]
impl<T: Rehydrate> ExactSizeIterator for RehydrateIter<'_, T> {}

/// Extension trait for Session to provide typed rehydration.
///
/// **Deprecated:** Use `Store` and `Table<T>` instead.
#[deprecated(since = "0.2.0", note = "Use Store and Table<T> instead")]
pub trait SessionRehydrateExt {
    /// Returns an iterator that lazily rehydrates all matches of a given type.
    #[allow(deprecated)]
    fn iter_rehydrated<T: Rehydrate>(&self) -> RehydrateIter<'_, T>;
}

/// A dehydrated reference to a match.
///
/// This is a lightweight handle that can be stored in DataFrames
/// and rehydrated on demand.
///
/// **Deprecated:** Use `Ref<T>` instead.
#[deprecated(since = "0.2.0", note = "Use Ref<T> instead")]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct MatchRef<T: Pattern> {
    /// Index into the query's result table.
    pub match_idx: u32,
    _marker: std::marker::PhantomData<T>,
}

#[allow(deprecated)]
impl<T: Pattern> MatchRef<T> {
    /// Creates a new match reference.
    pub fn new(match_idx: u32) -> Self {
        Self {
            match_idx,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the match index.
    pub fn index(&self) -> u32 {
        self.match_idx
    }
}

#[allow(deprecated)]
impl<T> MatchRef<T>
where
    T: Pattern,
    T::Match: Rehydrate,
{
    /// Rehydrates the referenced match.
    pub fn rehydrate(&self, ctx: &RehydrateContext<'_>) -> Result<T::Match, SessionError> {
        T::Match::rehydrate_by_index(self.match_idx, ctx)
    }
}

/// A dehydrated wire reference.
///
/// Stores just the cell ID instead of full CellInfo.
///
/// **Deprecated:** Use `CellId` instead.
#[deprecated(since = "0.2.0", note = "Use CellId instead")]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct WireRef {
    /// The cell ID, or None if unbound.
    pub cell_id: Option<u32>,
}

#[allow(deprecated)]
impl WireRef {
    /// Creates a new wire reference.
    pub fn new(cell_id: Option<u32>) -> Self {
        Self { cell_id }
    }

    /// Creates an unbound wire reference.
    pub fn unbound() -> Self {
        Self { cell_id: None }
    }

    /// Rehydrates to a full Wire<Match>.
    #[allow(deprecated)]
    pub fn rehydrate(&self, path: Instance, ctx: &RehydrateContext<'_>) -> Wire<Match> {
        ctx.rehydrate_wire(path, self.cell_id)
    }
}

#[allow(deprecated)]
impl From<Option<u32>> for WireRef {
    fn from(cell_id: Option<u32>) -> Self {
        Self::new(cell_id)
    }
}

#[allow(deprecated)]
impl From<u32> for WireRef {
    fn from(cell_id: u32) -> Self {
        Self::new(Some(cell_id))
    }
}
