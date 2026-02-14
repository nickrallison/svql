//! Typed table wrapper for pattern results.
//!
//! `Table<T>` wraps a simple columnar store while providing type-safe
//! access to rows as `Row<T>` and references as `Ref<T>`.

use std::marker::PhantomData;

use super::columnar::ColumnStore;
use crate::prelude::*;
use crate::wire::Wire;

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
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the row data is inconsistent (currently always returns `Ok`).
    pub fn new(rows: Vec<EntryArray>) -> Result<Self, QueryError>
    where
        T: Pattern,
    {
        let column_names: Vec<String> = T::schema()
            .columns()
            .iter()
            .map(|c| c.name.to_string())
            .collect();

        let mut store = ColumnStore::new(column_names);

        for row in rows {
            store.push_row(row);
        }

        Ok(Self {
            store,
            _marker: PhantomData,
        })
    }

    /// Deduplicate rows in the table.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if reconstruction of the table fails.
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
    pub const fn len(&self) -> usize {
        self.store.height()
    }

    /// Check if the table is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.store.height() == 0
    }

    /// Get a reference to the underlying columnar store for bulk operations.
    #[inline]
    pub const fn store(&self) -> &ColumnStore {
        &self.store
    }

    /// Get a mutable reference to the underlying columnar store.
    #[inline]
    pub const fn store_mut(&mut self) -> &mut ColumnStore {
        &mut self.store
    }

    /// Retrieves a specific entry from the table by row and column name.
    pub fn get_entry(&self, row_idx: usize, col_name: &str) -> ColumnEntry {
        self.store.get_cell(col_name, row_idx).clone()
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

        // Extract entries directly from the store
        for (idx, col) in T::schema().columns().iter().enumerate() {
            row.entry_array.entries[idx] = self.get_entry(row_idx_usize, col.name);
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
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if:
    /// * The file cannot be created.
    /// * Writing to the CSV file fails.
    /// * Flushing the writer fails.
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
                    match &col[row_idx] {
                        ColumnEntry::Null => record.push(String::new()),
                        ColumnEntry::Wire(wire_ref) => record.push(format!("{wire_ref:?}")),
                        ColumnEntry::WireArray(wires) => {
                            let wire_strs: Vec<String> = wires.iter().map(|w| format!("{w:?}")).collect();
                            record.push(format!("[{}]", wire_strs.join(", ")));
                        }
                        ColumnEntry::Sub(slot_idx) => record.push(format!("Ref({slot_idx})")),
                        ColumnEntry::Metadata(id) => record.push(format!("{id}")),
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
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if formatting the string fails.
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
                        match &col[row_idx] {
                            ColumnEntry::Null => record.push(String::new()),
                            ColumnEntry::Wire(wire_ref) => record.push(format!("{wire_ref:?}")),
                            ColumnEntry::WireArray(wires) => {
                                let wire_strs: Vec<String> = wires.iter().map(|w| format!("{w:?}")).collect();
                                record.push(format!("[{}]", wire_strs.join(", ")));
                            }
                            ColumnEntry::Sub(slot_idx) => record.push(format!("Ref({slot_idx})")),
                            ColumnEntry::Metadata(id) => record.push(format!("{id}")),
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
        writeln!(f, "\nTable: {}", type_name)?;
        writeln!(f, "Rows: {}", self.store.height())?;
        writeln!(f, "Columns: {}", self.store.column_names().len())?;
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

    /// Resolve a selector path within a specific row to a wire bundle.
    /// Used for set-based connectivity checking with `#[connect_any]`.
    fn resolve_bundle_path(
        &self,
        row_idx: usize,
        selector: crate::dsl::selector::Selector<'_>,
        ctx: &crate::session::ExecutionContext,
    ) -> Option<Vec<Wire>>;

    /// Export this table to a CSV file.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the CSV export fails.
    fn to_csv(&self, path: &std::path::Path) -> Result<(), QueryError>;
}

impl<T> AnyTable for Table<T>
where
    T: Pattern + Component + Send + Sync + 'static,
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

        match self.store.get_cell(col_name, row_idx) {
            ColumnEntry::Wire(WireRef::Cell(cid)) => Some(*cid),
            _ => None,
        }
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

        match self.store.get_cell(col_name, row_idx) {
            ColumnEntry::Sub(slot_idx) => Some((*slot_idx, target_type)),
            _ => None,
        }
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
        let sub_row_idx = match self.store.get_cell(head, row_idx) {
            ColumnEntry::Sub(slot_idx) => *slot_idx,
            _ => return None,
        };

        // Get the submodule's table and continue resolution
        let sub_table = ctx.get_any_table(sub_type_id)?;
        sub_table.resolve_path(sub_row_idx as usize, selector.tail(), ctx)
    }

    fn resolve_bundle_path(
        &self,
        row_idx: usize,
        selector: crate::dsl::selector::Selector<'_>,
        ctx: &crate::session::ExecutionContext,
    ) -> Option<Vec<Wire>> {
        if selector.is_empty() {
            return None;
        }

        // Single segment: direct wire bundle lookup
        if selector.len() == 1 {
            let col_name = selector.head()?;
            let col_idx = T::schema().index_of(col_name)?;
            
            // Check if it's a WireArray column
            if !T::schema().column(col_idx).kind.is_wire_array() {
                return None;
            }

            return match self.store.get_cell(col_name, row_idx) {
                ColumnEntry::WireArray(wire_refs) => {
                    let direction = T::schema().column(col_idx).direction;
                    Some(wire_refs.iter().map(|wr| Wire::from_ref(wr.clone(), direction)).collect())
                }
                _ => None,
            };
        }

        // Multi-segment: traverse through submodules
        let head = selector.head()?;
        let col_idx = T::schema().index_of(head)?;
        let col_def = T::schema().column(col_idx);

        // Head must be a submodule reference
        let sub_type_id = col_def.as_submodule()?;

        // Get the submodule row index directly from the column
        let sub_row_idx = match self.store.get_cell(head, row_idx) {
            ColumnEntry::Sub(slot_idx) => *slot_idx,
            _ => return None,
        };

        // Get the submodule's table and continue resolution
        let sub_table = ctx.get_any_table(sub_type_id)?;
        sub_table.resolve_bundle_path(sub_row_idx as usize, selector.tail(), ctx)
    }

    fn to_csv(&self, path: &std::path::Path) -> Result<(), QueryError> {
        Self::to_csv(self, path)
    }
}
