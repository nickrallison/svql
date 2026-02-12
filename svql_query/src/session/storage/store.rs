//! Central storage for pattern result tables.
//!
//! `Store` holds type-erased `Table<T>` instances, allowing patterns
//! to access their dependencies' results during search and rehydration.

use std::any::TypeId;
use std::sync::Arc;

use crate::prelude::*;

/// Central storage for all pattern result tables.
///
/// During execution, the `ExecutionPlan` populates the store with
/// `Table<T>` for each pattern type. Patterns can then access their
/// dependencies via `ctx.get::<Dep>()` during `search()`.
///
/// # Thread Safety
///
/// `Store` is `Send + Sync` and uses `OnceLock` slots internally during
/// execution to ensure each table is written exactly once.
///
/// # Usage
///
/// ```ignore
/// // During Pattern::search()
/// let dep_table = ctx.get::<Dep>().ok_or(QueryError::MissingDependency("Dep"))?;
///
/// // After execution
/// let store = plan.execute(&driver, key, config)?;
/// let table = store.get::<MyPattern>().unwrap();
/// for row in table.rows() {
///     let matched = MyPattern::rehydrate(&row, &store);
/// }
///
/// // Export all tables (including subtables) to CSV files
/// store.to_csv_dir("output/csv_results")?;
/// ```
pub struct Store {
    /// Type-erased table storage.
    tables: HashMap<TypeId, Arc<dyn AnyTable + Send + Sync>>,
    // cells: Vec<Cell>,
}

impl Store {
    /// Create an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            // cells: Vec::new(),
        }
    }

    /// Create a store with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tables: HashMap::with_capacity(capacity),
            // cells: Vec::new(),
        }
    }

    // pub fn add_cells(&mut self, new_cells: &[Cell]) {
    //     // self.cells.extend_from_slice(new_cells);
    // }

    // pub fn cells(&self) -> &[Cell] {
    //     &self.cells
    // }

    /// Return an iterator over all tables in the store.
    pub fn tables(&self) -> impl Iterator<Item = (&TypeId, &(dyn AnyTable + Send + Sync))> {
        self.tables
            .iter()
            .map(|(type_id, arc)| (type_id, arc.as_ref() as &(dyn AnyTable + Send + Sync)))
    }

    /// Insert a table for pattern type `T`.
    ///
    /// Overwrites any existing table for this type.
    pub fn insert<T: Send + Sync + Pattern + Component + 'static>(&mut self, table: Table<T>) {
        self.tables.insert(TypeId::of::<T>(), Arc::new(table));
    }

    /// Get a table for pattern type `T`.
    ///
    /// Returns `None` if no table exists for this type.
    #[must_use]
    pub fn get<T: 'static>(&self) -> Option<&Table<T>> {
        self.tables
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.as_any().downcast_ref::<Table<T>>())
    }

    /// Check if a table exists for pattern type `T`.
    #[must_use]
    pub fn contains<T: 'static>(&self) -> bool {
        self.tables.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of tables in the store.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tables.len()
    }

    /// Check if the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    #[must_use]
    /// Retrieves a type-erased table using a raw `TypeId`.
    pub fn get_from_tid(&self, type_id: TypeId) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.tables.get(&type_id).map(std::convert::AsRef::as_ref)
    }

    /// Resolve a reference to a row.
    ///
    /// This is a convenience method that combines `get()` and `Table::row()`.
    #[must_use]
    pub fn resolve<T>(&self, r: Ref<T>) -> Option<Row<T>>
    where
        T: crate::traits::Pattern + crate::traits::Component + 'static,
    {
        self.get::<T>().and_then(|table| table.row(r.index()))
    }

    /// Get an iterator over all `TypeIds` in the store.
    pub fn type_ids(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.tables.keys().copied()
    }

    /// Get a type-erased table by `TypeId`.
    ///
    /// This is useful for generic code that doesn't know the concrete type.
    #[must_use]
    pub fn get_any(&self, type_id: TypeId) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.tables.get(&type_id).map(std::convert::AsRef::as_ref)
    }

    /// Insert a type-erased table from a Box.
    ///
    /// This is used by the execution engine.
    pub fn insert_any(&mut self, type_id: TypeId, table: Box<dyn AnyTable + Send + Sync>) {
        self.tables.insert(type_id, Arc::from(table));
    }

    /// Insert a type-erased table from an Arc.
    ///
    /// This is used by the execution engine when transferring from `OnceLock` slots.
    pub fn insert_arc(&mut self, type_id: TypeId, table: Arc<dyn AnyTable + Send + Sync>) {
        self.tables.insert(type_id, table);
    }

    /// Export all tables in the store to CSV files in the specified directory.
    ///
    /// Each table is saved as a separate CSV file named after its type.
    /// Creates the output directory if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Directory path where CSV files will be written
    ///
    /// # Returns
    ///
    /// Returns the number of tables successfully exported, or an error if any export fails.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if:
    /// * The output directory cannot be created.
    /// * Any of the individual CSV exports fail.
    pub fn to_csv_dir<P: AsRef<std::path::Path>>(
        &self,
        output_dir: P,
    ) -> Result<usize, QueryError> {
        let output_path = output_dir.as_ref();

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_path).map_err(|e| {
            QueryError::ExecutionError(format!("Failed to create output directory: {}", e))
        })?;

        let mut count = 0;
        for type_id in self.type_ids() {
            if let Some(table) = self.get_any(type_id) {
                // Sanitize the type name to create a valid filename
                let type_name = table.type_name();
                let filename = sanitize_type_name_for_filename(type_name);
                let file_path = output_path.join(format!("{}.csv", filename));

                table.to_csv(&file_path)?;
                count += 1;
            }
        }

        Ok(count)
    }
}

/// Convert a Rust type name to a safe filename.
///
/// Replaces `::` with `_` and removes angle brackets and other special characters.
fn sanitize_type_name_for_filename(type_name: &str) -> String {
    type_name
        .replace("::", "_")
        .replace(['<', '>', ','], "_")
        .replace(' ', "")
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("num_tables", &self.tables.len())
            .finish()
    }
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\n╔══════════════════════════════════════════════════════════════════════════════"
        )?;
        writeln!(f, "║ Store: {} tables", self.tables.len())?;
        writeln!(
            f,
            "╚══════════════════════════════════════════════════════════════════════════════"
        )?;

        if self.is_empty() {
            writeln!(f, "(empty store)")?;
            return Ok(());
        }

        // Show summary table of contents
        for (idx, type_id) in self.type_ids().enumerate() {
            if let Some(table) = self.get_any(type_id) {
                let type_name = table.type_name();
                let row_count = table.len();
                writeln!(f, "  [{idx:2}] {type_name} - {row_count} rows")?;
            }
        }

        Ok(())
    }
}
