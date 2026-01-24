//! Owned row snapshot from a pattern's result table.
//!
//! `Row<T>` provides an owned copy of a single row's data, avoiding
//! lifetime complexity when iterating or passing rows around.

use std::fmt::Display;
use std::marker::PhantomData;

use crate::session::{EntryArray, QueryError};
use crate::traits::{Pattern, schema_lut};

use super::ref_type::Ref;

/// An owned snapshot of a single row from a `Table<T>`.
///
/// This is created by `Table::row()` or during iteration, and holds
/// all the data needed to reconstruct a `T::Match` via `Pattern::rehydrate()`.
#[derive(Debug, Clone)]
pub struct Row<T>
where
    T: Pattern + svql_query::traits::Component,
{
    /// Row index in the source table.
    pub(crate) idx: u64,
    /// Wire columns: name → CellId (None if NULL).
    pub(crate) entry_array: EntryArray,
    /// Type marker.
    pub(crate) _marker: PhantomData<T>,
}

impl<T> Row<T>
where
    T: Pattern + svql_query::traits::Component,
{
    /// Create a new row (typically called by Table).
    pub fn new(idx: u64) -> Self {
        Self {
            idx,
            entry_array: EntryArray::with_capacity(T::SCHEMA_SIZE),
            _marker: PhantomData,
        }
    }

    /// Get the row index in the source table.
    #[inline]
    pub fn index(&self) -> u64 {
        self.idx
    }

    /// Get this row as a typed reference.
    #[inline]
    pub fn as_ref(&self) -> Ref<T> {
        Ref::new(self.idx)
    }

    /// Get a wire reference by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    #[inline]
    pub fn wire(&self, name: &str) -> Option<u64> {
        let idx = schema_lut(name, T::SCHEMA)?;

        self.entry_array
            .entries
            .get(idx)
            .and_then(|entry| match entry {
                crate::session::ColumnEntry::Cell { id, .. } => *id,
                _ => None,
            })
    }

    /// Get a submodule reference by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    pub fn sub<S>(&self, name: &str) -> Option<Ref<S>> {
        let idx = schema_lut(name, T::SCHEMA)?;
        self.entry_array
            .entries
            .get(idx)
            .and_then(|entry| match entry {
                crate::session::ColumnEntry::Sub { id: Some(id), .. } => Some(Ref::new(*id)),
                _ => None,
            })
    }

    // --- Builder methods (used by Table when constructing rows) ---

    /// Set a wire column value.
    pub fn with_wire(mut self, name: &'static str, cell_id: u64) -> Result<Self, QueryError> {
        let id =
            schema_lut(name, T::SCHEMA).ok_or_else(|| QueryError::SchemaLut(name.to_string()))?;
        self.entry_array.entries[id] = crate::session::ColumnEntry::Cell { id: Some(cell_id) };
        Ok(self)
    }

    /// Set a submodule column value (with optional index).
    pub fn with_sub(mut self, name: &'static str, idx: Option<u64>) -> Self {
        let id = schema_lut(name, T::SCHEMA).expect("Schema LUT missing sub column");
        self.entry_array.entries[id] = crate::session::ColumnEntry::Sub { id: idx };
        self
    }
}

impl<T> Default for Row<T>
where
    T: Pattern + svql_query::traits::Component,
{
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> Display for Row<T>
where
    T: Pattern + svql_query::traits::Component,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Get the type name for the header
        let type_name = std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap_or("Row");

        writeln!(f, "{}[{}]:", type_name, self.idx)?;

        // Iterate through the schema and print each column
        for (idx, col_def) in T::SCHEMA.iter().enumerate() {
            let entry = self.entry_array.entries.get(idx);

            let value_str = match entry {
                Some(crate::session::ColumnEntry::Cell { id: Some(id) }) => {
                    format!("cell({})", id)
                }
                Some(crate::session::ColumnEntry::Cell { id: None }) => "cell=NULL".to_string(),
                Some(crate::session::ColumnEntry::Sub { id: Some(id) }) => {
                    format!("ref({})", id)
                }
                Some(crate::session::ColumnEntry::Sub { id: None }) => "ref=NULL".to_string(),
                Some(crate::session::ColumnEntry::Metadata { id: Some(id) }) => {
                    format!("meta({})", id)
                }
                Some(crate::session::ColumnEntry::Metadata { id: None }) => "meta=NULL".to_string(),
                None => "MISSING".to_string(),
            };

            let nullable_marker = if col_def.nullable { "?" } else { "" };
            writeln!(f, "  {}{}: {}", col_def.name, nullable_marker, value_str)?;
        }

        Ok(())
    }
}
