//! Central storage for pattern result tables.
//!
//! `Store` holds type-erased `Table<T>` instances, allowing patterns
//! to access their dependencies' results during search and rehydration.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use super::ref_type::Ref;
use super::row::Row;
use super::table::{AnyTable, Table};

/// Central storage for all pattern result tables.
///
/// During execution, the `ExecutionPlan` populates the store with
/// `Table<T>` for each pattern type. Patterns can then access their
/// dependencies via `ctx.get::<Dep<Search>>()` during `search()`.
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
/// let dep_table = ctx.get::<Dep<Search>>().ok_or(QueryError::MissingDependency("Dep"))?;
///
/// // After execution
/// let store = plan.execute(&driver, key, config)?;
/// let table = store.get::<MyPattern<Search>>().unwrap();
/// for row in table.rows() {
///     let matched = MyPattern::rehydrate(&row, &store);
/// }
/// ```
pub struct Store {
    /// Type-erased table storage.
    tables: HashMap<TypeId, Arc<dyn AnyTable>>,
}

impl Store {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Create a store with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tables: HashMap::with_capacity(capacity),
        }
    }

    /// Insert a table for pattern type `T`.
    ///
    /// Overwrites any existing table for this type.
    pub fn insert<T: Send + Sync + 'static>(&mut self, table: Table<T>) {
        self.tables.insert(TypeId::of::<T>(), Arc::new(table));
    }

    /// Get a table for pattern type `T`.
    ///
    /// Returns `None` if no table exists for this type.
    pub fn get<T: 'static>(&self) -> Option<&Table<T>> {
        self.tables
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.as_any().downcast_ref::<Table<T>>())
    }

    /// Check if a table exists for pattern type `T`.
    pub fn contains<T: 'static>(&self) -> bool {
        self.tables.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of tables in the store.
    pub fn len(&self) -> usize {
        self.tables.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    /// Resolve a reference to a row.
    ///
    /// This is a convenience method that combines `get()` and `Table::row()`.
    pub fn resolve<T: 'static>(&self, r: Ref<T>) -> Option<Row<T>> {
        self.get::<T>().and_then(|table| table.row(r.index()))
    }

    /// Get an iterator over all TypeIds in the store.
    pub fn type_ids(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.tables.keys().copied()
    }

    /// Get a type-erased table by TypeId.
    ///
    /// This is useful for generic code that doesn't know the concrete type.
    pub fn get_any(&self, type_id: TypeId) -> Option<&dyn AnyTable> {
        self.tables.get(&type_id).map(|arc| arc.as_ref())
    }

    /// Insert a type-erased table from a Box.
    ///
    /// This is used by the execution engine.
    pub fn insert_any(&mut self, type_id: TypeId, table: Box<dyn AnyTable>) {
        self.tables.insert(type_id, Arc::from(table));
    }

    /// Insert a type-erased table from an Arc.
    ///
    /// This is used by the execution engine when transferring from OnceLock slots.
    pub fn insert_arc(&mut self, type_id: TypeId, table: Arc<dyn AnyTable>) {
        self.tables.insert(type_id, table);
    }
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
                writeln!(f, "  [{:2}] {} - {} rows", idx, type_name, row_count)?;
            }
        }

        Ok(())
    }
}

// Store is Send + Sync because all tables are Send + Sync
unsafe impl Send for Store {}
unsafe impl Sync for Store {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::cell_id::CellId;
    use crate::session::column::ColumnDef;
    use crate::session::table::TableBuilder;

    struct PatternA;
    struct PatternB;

    static COLUMNS_A: &[ColumnDef] = &[ColumnDef::wire("wire_a")];
    static COLUMNS_B: &[ColumnDef] = &[ColumnDef::wire("wire_b")];

    #[test]
    fn test_store_insert_get() {
        let mut store = Store::new();

        // Build a table for PatternA
        let mut builder_a: TableBuilder<PatternA> = TableBuilder::new(COLUMNS_A);
        builder_a.push(
            Row::<PatternA>::new(0, "path_a".to_string())
                .with_wire("wire_a", Some(CellId::new(100))),
        );
        let table_a = builder_a.build().unwrap();

        store.insert(table_a);

        // Should be able to get PatternA
        assert!(store.get::<PatternA>().is_some());
        assert!(store.contains::<PatternA>());

        // Should not have PatternB
        assert!(store.get::<PatternB>().is_none());
        assert!(!store.contains::<PatternB>());
    }

    #[test]
    fn test_store_resolve() {
        let mut store = Store::new();

        let mut builder: TableBuilder<PatternA> = TableBuilder::new(COLUMNS_A);
        builder.push(
            Row::<PatternA>::new(0, "path_0".to_string())
                .with_wire("wire_a", Some(CellId::new(42))),
        );
        builder.push(
            Row::<PatternA>::new(1, "path_1".to_string())
                .with_wire("wire_a", Some(CellId::new(43))),
        );
        store.insert(builder.build().unwrap());

        let r: Ref<PatternA> = Ref::new(1);
        let row = store.resolve(r).unwrap();
        assert_eq!(row.path(), "path_1");
        assert_eq!(row.wire("wire_a"), Some(CellId::new(43)));

        // Out of bounds
        let bad: Ref<PatternA> = Ref::new(999);
        assert!(store.resolve(bad).is_none());
    }

    #[test]
    fn test_store_multiple_types() {
        let mut store = Store::new();

        let mut builder_a: TableBuilder<PatternA> = TableBuilder::new(COLUMNS_A);
        builder_a.push(Row::<PatternA>::new(0, "a".to_string()));
        store.insert(builder_a.build().unwrap());

        let mut builder_b: TableBuilder<PatternB> = TableBuilder::new(COLUMNS_B);
        builder_b.push(Row::<PatternB>::new(0, "b1".to_string()));
        builder_b.push(Row::<PatternB>::new(1, "b2".to_string()));
        store.insert(builder_b.build().unwrap());

        assert_eq!(store.len(), 2);
        assert_eq!(store.get::<PatternA>().unwrap().len(), 1);
        assert_eq!(store.get::<PatternB>().unwrap().len(), 2);
    }
}
