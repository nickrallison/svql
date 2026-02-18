use super::AnyTable;
use super::Ref;
use super::Row;
use super::Table;
use crate::traits::Component;
use crate::traits::Pattern;
use gxhash::{HashMap, HashMapExt};
use std::any::TypeId;
use std::sync::Arc;

/// A store containing multiple tables, indexed by type ID.
pub struct Store {
    /// Map from type ID to table.
    tables: HashMap<TypeId, Arc<dyn AnyTable + Send + Sync>>,
}

impl Store {
    /// Create a new empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Create a new store with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tables: HashMap::with_capacity(capacity),
        }
    }

    /// Iterate over all tables in the store.
    pub fn tables(&self) -> impl Iterator<Item = (&TypeId, &(dyn AnyTable + Send + Sync))> {
        self.tables.iter().map(|(tid, arc)| (tid, arc.as_ref()))
    }

    /// Iterate over all type IDs in the store.
    pub fn type_ids(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.tables.keys().copied()
    }

    /// Insert a table into the store.
    pub fn insert<T: Send + Sync + Pattern + Component + 'static>(&mut self, table: Table<T>) {
        self.tables.insert(TypeId::of::<T>(), Arc::new(table));
    }

    /// Insert a type-erased table using a type ID.
    pub fn insert_arc(&mut self, type_id: TypeId, table: Arc<dyn AnyTable + Send + Sync>) {
        self.tables.insert(type_id, table);
    }

    /// Get a typed table from the store.
    #[must_use]
    pub fn get<T: 'static>(&self) -> Option<&Table<T>> {
        self.tables
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.as_any().downcast_ref::<Table<T>>())
    }

    /// Resolve a reference to a row in the store.
    pub fn resolve<T>(&self, r: Ref<T>) -> Option<Row<T>>
    where
        T: 'static + Pattern + Component,
    {
        self.get::<T>().and_then(|table| table.row(r))
    }

    /// Get a type-erased table by type ID.
    #[must_use]
    pub fn get_any(&self, type_id: TypeId) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.tables.get(&type_id).map(|arc| arc.as_ref())
    }

    /// Get a type-erased table by type ID (alias for `get_any`).
    #[must_use]
    pub fn get_from_tid(&self, type_id: TypeId) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.get_any(type_id)
    }

    /// Get the number of tables in the store.
    pub fn len(&self) -> usize {
        self.tables.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    /// Clear all tables from the store.
    pub fn clear(&mut self) {
        self.tables.clear();
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
