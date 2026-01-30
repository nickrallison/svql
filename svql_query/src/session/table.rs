//! Typed DataFrame wrapper for pattern results.
//!
//! `Table<T>` wraps a Polars DataFrame while providing type-safe
//! access to rows as `Row<T>` and references as `Ref<T>`.

use std::marker::PhantomData;

use polars::prelude::*;

use crate::session::{ColumnEntry, EntryArray, Row};
use crate::traits::{Component, Pattern};

use super::error::QueryError;
use super::ref_type::Ref;
// use super::row::Row;
use super::schema::ColumnKind;

/// A typed wrapper around a DataFrame storing pattern match results.
///
/// Each row in the DataFrame represents one match of pattern `T`.
/// The schema is defined by `T::COLUMNS` on the `Pattern` trait.
///
/// # Column Types
///
/// - **Wire columns** (`ColumnKind::Wire`): Store as `i64` (CellId::raw())
/// - **Sub columns** (`ColumnKind::Sub`): Store as `u32` (row index or -1 for NULL)
/// - **Metadata columns**: Various types (depth as u32, etc.)
/// - **path**: Always present as `Utf8` column
///
/// # Usage
///
/// ```ignore
/// let table: &Table<MyPattern<Search>> = store.get()?;
/// for row in table.rows() {
///     let wire = row.wire("clk")?;
///     let sub: Ref<Dep<Search>> = row.sub("dep")?;
/// }
/// ```
pub struct Table<T> {
    /// The underlying Polars DataFrame.
    df: DataFrame,
    // /// Column schema for this pattern type.
    // columns: &'static [ColumnDef],
    // /// Mapping from submodule column names to their target TypeId.
    // /// Used for runtime type checking when accessing sub columns.
    // sub_types: HashMap<&'static str, TypeId>,
    /// Type marker.
    _marker: PhantomData<T>,
}

impl<T> Table<T>
where
    T: Pattern + Component,
{
    /// Create a table from a pre-collected set of row matches.
    ///
    /// This is efficient as it performs a single allocation per column.
    pub fn new(rows: Vec<EntryArray>) -> Result<Self, QueryError>
    where
        T: Pattern,
    {
        if rows.is_empty() {
            let cols: Vec<Column> = T::schema()
                .columns()
                .iter()
                .map(|col_def| col_def.into_polars_column())
                .collect();
            let df = DataFrame::new(cols)?;
            return Ok(Self {
                df,
                _marker: PhantomData,
            });
        }

        let mut columns = Vec::with_capacity(T::SCHEMA_SIZE + 1);

        // 1. Handle structural columns from the RowMatch array
        for i in 0..T::SCHEMA_SIZE {
            let col_def = T::schema().column(i);
            let series_data: Vec<Option<u64>> =
                rows.iter().map(|r| r.entries[i].as_u64()).collect();
            columns.push(Column::new(
                PlSmallStr::from_static(col_def.name),
                series_data,
            ));
        }

        let df = DataFrame::new(columns)?;
        Ok(Self {
            df,
            _marker: PhantomData,
        })
    }

    /// Deduplicate rows in the table.
    pub fn deduplicate(&self) -> Result<Self, QueryError> {
        let subset: Vec<String> = T::schema()
            .columns()
            .iter()
            .map(|c| c.name.to_string())
            .collect();

        let df = self.df.clone().unique::<Vec<String>, String>(
            Some(&subset),
            UniqueKeepStrategy::First,
            None,
        )?;

        Ok(Self {
            df,
            _marker: PhantomData,
        })
    }

    /// Get the number of rows (matches) in this table.
    #[inline]
    pub fn len(&self) -> usize {
        self.df.height()
    }

    /// Check if the table is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.df.height() == 0
    }

    /// Get a reference to the underlying DataFrame for bulk operations.
    #[inline]
    pub fn df(&self) -> &DataFrame {
        &self.df
    }

    /// Get a mutable reference to the underlying DataFrame.
    #[inline]
    pub fn df_mut(&mut self) -> &mut DataFrame {
        &mut self.df
    }

    pub fn get_entry(&self, row_idx: usize, col_name: &str) -> Option<ColumnEntry> {
        let col = self.df.column(col_name).ok()?;
        let idx = T::schema().index_of(col_name)?;
        match T::schema().column(idx).kind {
            ColumnKind::Cell => {
                let ca = col.u64().ok()?;
                let val = ca.get(row_idx).map(|raw| raw);
                Some(ColumnEntry::Cell { id: val })
            }
            ColumnKind::Sub(_) => {
                let ca = col.u64().ok()?;
                let val = ca.get(row_idx).map(|raw| raw);
                Some(ColumnEntry::Sub { id: val })
            }
            ColumnKind::Metadata => {
                let ca = col.u64().ok()?;
                let val = ca.get(row_idx).map(|raw| raw);
                Some(ColumnEntry::Metadata { id: val })
            }
        }
    }

    /// Get a single row by index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn row(&self, row_idx: u64) -> Option<Row<T>> {
        let row_idx_usize = row_idx as usize;
        if row_idx_usize >= self.df.height() {
            return None;
        }

        let mut row = Row::new(row_idx);

        // Extract wire and sub columns
        for (idx, col) in T::schema().columns().iter().enumerate() {
            row.entry_array.entries[idx] = self.get_entry(row_idx_usize, col.name)?;
        }

        Some(row)
    }

    /// Iterate over all rows in the table.
    pub fn rows(&self) -> impl Iterator<Item = Row<T>> + '_ {
        (0..self.len() as u64).filter_map(|idx| self.row(idx))
    }

    /// Iterate over references to all rows.
    pub fn refs(&self) -> impl Iterator<Item = Ref<T>> {
        (0..self.len() as u64).map(Ref::new)
    }

    // /// Get the TypeId for a submodule column.
    // pub fn sub_type(&self, name: &str) -> Option<TypeId> {
    //     self.sub_types.get(name).copied()
    // }
}

impl<T> std::fmt::Debug for Table<T>
where
    T: Pattern + Component,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table")
            .field("type", &std::any::type_name::<T>())
            .field("len", &self.len())
            .field("columns", &T::schema().columns())
            .field("df", &self.df)
            .finish()
    }
}

impl<T> std::fmt::Display for Table<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = std::any::type_name::<T>();
        writeln!(
            f,
            "\n╔══════════════════════════════════════════════════════════════════════════════"
        )?;
        writeln!(f, "║ Table: {}  ", type_name)?;
        writeln!(f, "║ Rows: {}", self.df.height())?;
        writeln!(
            f,
            "╚══════════════════════════════════════════════════════════════════════════════"
        )?;

        // Use Polars' built-in DataFrame display with row index for easier reference
        let df_with_row = self
            .df
            .clone()
            .with_row_index("row".into(), None)
            .unwrap_or(self.df.clone());
        write!(f, "{}", df_with_row)?;

        Ok(())
    }
}

/// Type-erased table trait for storing in `Store`.
pub trait AnyTable: Send + Sync + std::fmt::Display + 'static {
    /// Downcast to concrete type.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get the number of rows.
    fn len(&self) -> usize;

    /// Check if empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the type name of the table.
    fn type_name(&self) -> &str;

    /// Get the TypeId of the pattern stored in this table.
    fn pattern_type_id(&self) -> std::any::TypeId;

    /// Get a submodule reference (Row Index + TypeId) for a given column.
    fn get_sub_ref(&self, row_idx: usize, col_name: &str) -> Option<(u64, std::any::TypeId)>;

    /// Get a cell ID by single column name (no path traversal).
    fn get_cell_id(&self, row_idx: usize, col_name: &str) -> Option<u64>;

    /// Resolve a path (e.g., "and1.y" or just "y") to a cell ID.
    ///
    /// This method traverses through submodule references to resolve deep paths.
    /// For single-segment paths, it's equivalent to `get_cell_id`.
    ///
    /// # Arguments
    /// * `row_idx` - The row index in this table
    /// * `path` - A dot-separated path like "submod.port" or just "port"
    /// * `ctx` - The execution context (needed to fetch submodule tables)
    fn get_cell_id_path(
        &self,
        row_idx: usize,
        path: &str,
        ctx: &super::ExecutionContext,
    ) -> Option<u64>;
}

impl<T: Send + Sync + 'static> AnyTable for Table<T>
where
    T: Pattern + Component,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn len(&self) -> usize {
        self.df.height()
    }

    fn type_name(&self) -> &str {
        std::any::type_name::<T>()
    }

    fn get_cell_id(&self, row_idx: usize, col_name: &str) -> Option<u64> {
        // O(1) lookup
        let col_idx = T::schema().index_of(col_name)?;

        // Check if it is actually a cell column
        if !T::schema().column(col_idx).kind.is_wire() {
            return None;
        }

        let col = self.df.column(col_name).ok()?;
        let ca = col.u64().ok()?;
        ca.get(row_idx).map(|raw| raw)
    }

    fn pattern_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<T>()
    }

    fn get_sub_ref(&self, row_idx: usize, col_name: &str) -> Option<(u64, std::any::TypeId)> {
        let col_idx = T::schema().index_of(col_name)?;
        let col_def = T::schema().column(col_idx);
        let target_type = col_def.as_submodule()?;

        let col = self.df.column(col_name).ok()?;
        let ca = col.u64().ok()?;
        let val = ca.get(row_idx)?;

        Some((val, target_type))
    }

    fn get_cell_id_path(
        &self,
        row_idx: usize,
        path: &str,
        ctx: &super::ExecutionContext,
    ) -> Option<u64> {
        let segments: Vec<&str> = path.split('.').collect();

        if segments.is_empty() {
            return None;
        }

        // Single segment: just do a direct cell lookup
        if segments.len() == 1 {
            return self.get_cell_id(row_idx, segments[0]);
        }

        // Multi-segment path: traverse through submodules
        let head = segments[0];

        // Get the submodule reference for the head
        let (mut current_row_idx, mut current_type_id) = self.get_sub_ref(row_idx, head)?;

        // Traverse intermediate segments
        for (i, segment) in segments[1..].iter().enumerate() {
            let table = ctx.get_any_table(current_type_id)?;

            if i == segments.len() - 2 {
                // Last segment: must be a Cell
                return table.get_cell_id(current_row_idx as usize, segment);
            } else {
                // Intermediate segment: must be a Submodule
                let (next_idx, next_tid) = table.get_sub_ref(current_row_idx as usize, segment)?;
                current_row_idx = next_idx;
                current_type_id = next_tid;
            }
        }

        None
    }
}
