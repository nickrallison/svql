use super::AnyTable;
use super::Ref;
use super::Row;
use super::Table;
use crate::session::error::QueryError;
use crate::traits::Component;
use crate::traits::Pattern;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Store {
    tables: HashMap<TypeId, Arc<dyn AnyTable + Send + Sync>>,
}

impl Store {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tables: HashMap::with_capacity(capacity),
        }
    }

    pub fn tables(&self) -> impl Iterator<Item = (&TypeId, &(dyn AnyTable + Send + Sync))> {
        self.tables.iter().map(|(tid, arc)| (tid, arc.as_ref()))
    }

    pub fn type_ids(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.tables.keys().copied()
    }

    pub fn insert<T: Send + Sync + Pattern + Component + 'static>(&mut self, table: Table<T>) {
        self.tables.insert(TypeId::of::<T>(), Arc::new(table));
    }

    pub fn insert_arc(&mut self, type_id: TypeId, table: Arc<dyn AnyTable + Send + Sync>) {
        self.tables.insert(type_id, table);
    }

    #[must_use]
    pub fn get<T: 'static>(&self) -> Option<&Table<T>> {
        self.tables
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.as_any().downcast_ref::<Table<T>>())
    }

    pub fn resolve<T: 'static>(&self, r: Ref<T>) -> Option<Row<T>>
    where
        T: Pattern + Component,
    {
        self.get::<T>().and_then(|table| table.row(r))
    }

    #[must_use]
    pub fn get_any(&self, type_id: TypeId) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.tables.get(&type_id).map(|arc| arc.as_ref())
    }

    #[must_use]
    pub fn get_from_tid(&self, type_id: TypeId) -> Option<&(dyn AnyTable + Send + Sync)> {
        self.get_any(type_id)
    }

    pub fn len(&self) -> usize {
        self.tables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

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
