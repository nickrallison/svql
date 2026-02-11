//! Typed table wrapper for pattern results.
//!
//! `Table<T>` wraps a simple columnar store while providing type-safe
//! access to rows as `Row<T>` and references as `Ref<T>`.

use std::marker::PhantomData;

use super::columnar::ColumnStore;
use crate::prelude::*;
use crate::session::schema::SlotIdx;

/// A typed wrapper around a columnar store for pattern match results.
///
/// Each row represents one match of pattern `T`.
/// The schema is defined by `T::COLUMNS` on the `Pattern` trait.
///
/// # Column Types
///
/// - **Wire columns** (`ColumnKind::Wire`): Store as `u32` (`PhysicalCellId::raw()`)
/// - **Sub columns** (`ColumnKind::Sub`): Store as `u32` (row index)
/// - **Metadata columns**: Store as `u32`
///
/// # Usage
///
/// ```ignore
/// let table: &Table<MyPattern> = store.get()?;
/// for row in table.rows() {
///     let wire = row.wire("clk")?;
///     let sub: Ref<Dep> = row.sub("dep")?;
/// }
/// ```
pub struct Table<T> {
    /// The underlying columnar storage.
    store: ColumnStore,
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
        let column_names: Vec<String> = T::schema()
            .columns()
            .iter()
            .map(|c| c.name.to_string())
            .collect();

        if rows.is_empty() {
            let store = ColumnStore::new(column_names);
            return Ok(Self {
                store,
                _marker: PhantomData,
            });
        }

        let schema_size = T::schema().defs.len();
        let mut columns_data = Vec::with_capacity(schema_size);

        // 1. Handle structural columns from the RowMatch array
        for i in 0..schema_size {
            let series_data: Vec<Option<u32>> =
                rows.iter().map(|r| r.entries[i].as_u32()).collect();
            columns_data.push(series_data);
        }

        let store = ColumnStore::from_columns(column_names, columns_data)
            .map_err(QueryError::ExecutionError)?;

        Ok(Self {
            store,
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

        let store = self.store.deduplicate_subset(&subset);

        Ok(Self {
            store,
            _marker: PhantomData,
        })
    }

    /// Get the number of rows (matches) in this table.
    #[inline]
    pub fn len(&self) -> usize {
        self.store.height()
    }

    /// Check if the table is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.store.height() == 0
    }

    /// Get a reference to the underlying columnar store for bulk operations.
    #[inline]
    pub const fn store(&self) -> &ColumnStore {
        &self.store
    }

    /// Get a mutable reference to the underlying columnar store.
    #[inline]
    pub fn store_mut(&mut self) -> &mut ColumnStore {
        &mut self.store
    }

    pub fn get_entry(&self, row_idx: usize, col_name: &str) -> Option<ColumnEntry> {
        let col = self.store.column(col_name)?;
        let idx = T::schema().index_of(col_name)?;
        let val = col.get(row_idx).copied().flatten();

        match T::schema().column(idx).kind {
            ColumnKind::Cell => Some(ColumnEntry::Wire {
                value: val
                    .map(|v| PhysicalCellId::new(v as u32))
                    .map(crate::wire::WireRef::Cell),
            }),
            ColumnKind::Sub(_) => Some(ColumnEntry::Sub { id: val.map(SlotIdx::new) }),
            ColumnKind::Metadata => Some(ColumnEntry::Metadata { id: val.map(PhysicalCellId::new) }),
        }
    }

    /// Get a single row by index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn row(&self, row_idx: u32) -> Option<Row<T>> {
        let row_idx_usize = row_idx as usize;
        if row_idx_usize >= self.store.height() {
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
        (0..self.len() as u32).filter_map(|idx| self.row(idx))
    }

    /// Iterate over references to all rows.
    pub fn refs(&self) -> impl Iterator<Item = Ref<T>> {
        (0..self.len() as u32).map(Ref::new)
    }

    /// Export this table to a CSV file.
    ///
    /// Writes all rows to the specified file path with a header row.
    /// Values are written as integers, with NULL represented as empty cells.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let store = plan.execute(&driver, key, config)?;
    /// let table = store.get::<MyPattern>().unwrap();
    /// table.to_csv("results/my_pattern.csv")?;
    /// ```
    pub fn to_csv<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), QueryError> {
        let mut writer = csv::Writer::from_path(path).map_err(|e| {
            QueryError::ExecutionError(format!("Failed to create CSV writer: {}", e))
        })?;

        // Write header
        writer
            .write_record(self.store.column_names())
            .map_err(|e| {
                QueryError::ExecutionError(format!("Failed to write CSV header: {}", e))
            })?;

        // Write data rows
        for row_idx in 0..self.len() {
            let mut record = Vec::with_capacity(self.store.column_names().len());
            for col_name in self.store.column_names() {
                if let Some(col) = self.store.column(col_name) {
                    if let Some(Some(value)) = col.get(row_idx) {
                        record.push(value.to_string());
                    } else {
                        record.push(String::new()); // NULL as empty string
                    }
                }
            }
            writer.write_record(&record).map_err(|e| {
                QueryError::ExecutionError(format!("Failed to write CSV row: {}", e))
            })?;
        }

        writer.flush().map_err(|e| {
            QueryError::ExecutionError(format!("Failed to flush CSV writer: {}", e))
        })?;

        Ok(())
    }

    /// Export this table to a CSV string.
    ///
    /// Returns the CSV as a String instead of writing to a file.
    pub fn to_csv_string(&self) -> Result<String, QueryError> {
        let mut buffer = Vec::new();
        {
            let mut writer = csv::Writer::from_writer(&mut buffer);

            // Write header
            writer
                .write_record(self.store.column_names())
                .map_err(|e| {
                    QueryError::ExecutionError(format!("Failed to write CSV header: {}", e))
                })?;

            // Write data rows
            for row_idx in 0..self.len() {
                let mut record = Vec::with_capacity(self.store.column_names().len());
                for col_name in self.store.column_names() {
                    if let Some(col) = self.store.column(col_name) {
                        if let Some(Some(value)) = col.get(row_idx) {
                            record.push(value.to_string());
                        } else {
                            record.push(String::new()); // NULL as empty string
                        }
                    }
                }
                writer.write_record(&record).map_err(|e| {
                    QueryError::ExecutionError(format!("Failed to write CSV row: {}", e))
                })?;
            }

            writer.flush().map_err(|e| {
                QueryError::ExecutionError(format!("Failed to flush CSV writer: {}", e))
            })?;
        } // writer is dropped here, releasing the borrow on buffer

        String::from_utf8(buffer).map_err(|e| {
            QueryError::ExecutionError(format!("Failed to convert CSV to string: {}", e))
        })
    }
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
            .field("store", &self.store)
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
        writeln!(f, "║ Table: {type_name}  ")?;
        writeln!(f, "║ Rows: {}", self.store.height())?;
        writeln!(
            f,
            "╚══════════════════════════════════════════════════════════════════════════════"
        )?;

        write!(f, "{}", self.store)?;

        Ok(())
    }
}

/// Type-erased table trait for storing in `Store`.
pub trait AnyTable: Send + Sync + std::fmt::Display + 'static {
    /// Create a report node for a row (type-erased dispatch)
    fn row_to_report_node(
        &self,
        row_idx: usize,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<crate::traits::display::ReportNode>;

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

    /// Get the `TypeId` of the pattern stored in this table.
    fn pattern_type_id(&self) -> std::any::TypeId;

    /// Get a submodule reference (Row Index + `TypeId`) for a given column.
    fn get_sub_ref(&self, row_idx: usize, col_name: &str) -> Option<(u32, std::any::TypeId)>;

    /// Get a cell ID by single column name (no path traversal).
    fn get_cell_id(&self, row_idx: usize, col_name: &str) -> Option<PhysicalCellId>;

    /// Resolve a selector path within a specific row to a cell ID
    fn resolve_path(
        &self,
        row_idx: usize,
        selector: crate::dsl::selector::Selector<'_>,
        ctx: &crate::session::ExecutionContext,
    ) -> Option<PhysicalCellId>;

    /// Export this table to a CSV file.
    fn to_csv(&self, path: &std::path::Path) -> Result<(), QueryError>;
}

impl<T: Send + Sync + 'static> AnyTable for Table<T>
where
    T: Pattern + Component,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn len(&self) -> usize {
        self.store.height()
    }

    fn type_name(&self) -> &str {
        std::any::type_name::<T>()
    }

    fn get_cell_id(&self, row_idx: usize, col_name: &str) -> Option<PhysicalCellId> {
        // O(1) lookup
        let col_idx = T::schema().index_of(col_name)?;

        // Check if it is actually a cell column
        if !T::schema().column(col_idx).kind.is_wire() {
            return None;
        }

        let col = self.store.column(col_name)?;
        col.get(row_idx)
            .copied()
            .flatten()
            .map(|v| PhysicalCellId::new(v as u32))
    }

    fn pattern_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<T>()
    }

    fn row_to_report_node(
        &self,
        row_idx: usize,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<crate::traits::display::ReportNode> {
        let row = self.row(row_idx as u32)?;
        Some(T::row_to_report_node(&row, store, driver, key))
    }

    fn get_sub_ref(&self, row_idx: usize, col_name: &str) -> Option<(u32, std::any::TypeId)> {
        let col_idx = T::schema().index_of(col_name)?;
        let col_def = T::schema().column(col_idx);
        let target_type = col_def.as_submodule()?;

        let col = self.store.column(col_name)?;
        let val = col.get(row_idx).copied().flatten()?;

        Some((val, target_type))
    }

    fn resolve_path(
        &self,
        row_idx: usize,
        selector: crate::dsl::selector::Selector<'_>,
        ctx: &crate::session::ExecutionContext,
    ) -> Option<PhysicalCellId> {
        // Optimized: avoid allocating full Row<T> by directly accessing columns
        if selector.is_empty() {
            return None;
        }

        // Single segment: direct wire lookup
        if selector.len() == 1 {
            return self.get_cell_id(row_idx, selector.head()?);
        }

        // Multi-segment: traverse through submodules
        let head = selector.head()?;
        let col_idx = T::schema().index_of(head)?;
        let col_def = T::schema().column(col_idx);

        // Head must be a submodule reference
        let sub_type_id = col_def.as_submodule()?;

        // Get the submodule row index directly from the column
        let col = self.store.column(head)?;
        let sub_row_idx = col.get(row_idx).copied().flatten()?;

        // Get the submodule's table and continue resolution
        let sub_table = ctx.get_any_table(sub_type_id)?;
        sub_table.resolve_path(sub_row_idx as usize, selector.tail(), ctx)
    }

    fn to_csv(&self, path: &std::path::Path) -> Result<(), QueryError> {
        Table::<T>::to_csv(self, path)
    }
}
