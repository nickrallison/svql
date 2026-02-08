//! Owned row snapshot from a pattern's result table.
//!
//! `Row<T>` provides an owned copy of a single row's data, avoiding
//! lifetime complexity when iterating or passing rows around.

use std::fmt::Display;
use std::marker::PhantomData;

use crate::prelude::*;

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
    pub(crate) idx: u32,
    /// Wire columns: name → `CellId` (None if NULL).
    pub(crate) entry_array: EntryArray,
    /// Type marker.
    pub(crate) _marker: PhantomData<T>,
}

impl<T> Row<T>
where
    T: Pattern + svql_query::traits::Component,
{
    /// Create a new row (typically called by Table).
    #[must_use]
    pub fn new(idx: u32) -> Self {
        Self {
            idx,
            entry_array: EntryArray::with_capacity(T::schema().defs.len()),
            _marker: PhantomData,
        }
    }

    /// Get the row index in the source table.
    #[inline]
    #[must_use]
    pub const fn index(&self) -> u32 {
        self.idx
    }

    /// Get this row as a typed reference.
    #[inline]
    #[must_use]
    pub const fn as_ref(&self) -> Ref<T> {
        Ref::new(self.idx)
    }

    #[inline]
    #[must_use]
    pub fn entry_array(&self) -> &EntryArray {
        &self.entry_array
    }

    /// Get a wire reference by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    #[inline]
    #[must_use]
    pub fn wire(&self, name: &str) -> Option<Wire> {
        let idx = T::schema().index_of(name)?;
        let col_def = T::schema().column(idx);

        self.entry_array
            .entries
            .get(idx)
            .and_then(|entry| match entry {
                crate::session::ColumnEntry::Wire { value } => value.clone(),
                _ => None,
            })
            .map(|wire_ref| Wire::from_ref(wire_ref, col_def.direction))
    }

    /// Get a submodule reference by name (type-erased)
    pub fn sub_any(&self, name: &str) -> Option<u32> {
        let idx = T::schema().index_of(name)?;
        self.entry_array
            .entries
            .get(idx)
            .and_then(|entry| match entry {
                ColumnEntry::Sub { id } => *id,
                _ => None,
            })
    }

    /// Get all internal cell mappings from metadata columns.
    ///
    /// Returns an iterator of `(needle_debug_index, haystack_cell_id)` pairs.
    /// Only works for Netlist patterns with discovered internal cells.
    pub fn internal_cell_mappings(&self) -> impl Iterator<Item = (usize, u32)> + '_ {
        T::schema()
            .columns()
            .iter()
            .enumerate()
            .filter_map(|(idx, col_def)| {
                if !col_def.name.starts_with("__internal_cell_") {
                    return None;
                }

                let needle_debug_id: usize = col_def
                    .name
                    .strip_prefix("__internal_cell_")
                    .and_then(|s| s.parse().ok())?;

                let haystack_cell_id = self.entry_array.entries.get(idx)?.as_u32()?;

                Some((needle_debug_id, haystack_cell_id))
            })
    }

    /// Resolve a path to a wire using a Selector
    ///
    /// # Examples
    /// ```ignore
    /// use svql_query::selector::Selector;
    ///
    /// let sel = Selector::new(&["y"]);
    /// row.resolve(sel, ctx);  // Direct port
    ///
    /// let sel = Selector::new(&["and1", "y"]);
    /// row.resolve(sel, ctx);  // Submodule port
    /// ```
    #[must_use]
    pub fn resolve(
        &self,
        selector: crate::dsl::selector::Selector<'_>,
        ctx: &crate::session::ExecutionContext,
    ) -> Option<Wire> {
        if selector.is_empty() {
            return None;
        }

        // Single segment: direct lookup
        if selector.len() == 1 {
            return self.wire(selector.head()?);
        }

        // Multi-segment: traverse through submodules
        let head = selector.head()?;
        let idx = T::schema().index_of(head)?;
        let col_def = T::schema().column(idx);

        // Head must be a submodule reference
        let (sub_row_idx, sub_type_id) = match &col_def.kind {
            crate::session::ColumnKind::Sub(tid) => {
                let entry = self.entry_array.entries.get(idx)?;
                match entry {
                    crate::session::ColumnEntry::Sub { id: Some(id) } => (*id, *tid),
                    _ => return None,
                }
            }
            _ => return None,
        };

        // Get the submodule's table and continue resolution
        let sub_table = ctx.get_any_table(sub_type_id)?;
        let cell_id = sub_table.resolve_path(sub_row_idx as usize, selector.tail(), ctx)?;

        // Direction is lost through traversal
        Some(Wire::new(cell_id, PortDirection::None))
    }

    /// Get a submodule reference by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    #[must_use]
    pub fn sub<S>(&self, name: &str) -> Option<Ref<S>> {
        let idx = T::schema().index_of(name)?;
        self.entry_array
            .entries
            .get(idx)
            .and_then(|entry| match entry {
                crate::session::ColumnEntry::Sub { id: Some(id), .. } => Some(Ref::new(*id)),
                _ => None,
            })
    }

    // --- Builder methods (used by Table when constructing rows) ---

    /// Set a wire column value using a WireRef.
    pub fn with_wire_ref(
        mut self,
        name: &'static str,
        wire_ref: crate::wire::WireRef,
    ) -> Result<Self, QueryError> {
        let id = T::schema()
            .index_of(name)
            .ok_or_else(|| QueryError::SchemaLut(name.to_string()))?;
        self.entry_array.entries[id] = crate::session::ColumnEntry::Wire {
            value: Some(wire_ref),
        };
        Ok(self)
    }

    /// Set a wire column value with a cell ID (legacy method).
    pub fn with_wire(self, name: &'static str, cell_id: CellId) -> Result<Self, QueryError> {
        self.with_wire_ref(name, crate::wire::WireRef::Cell(cell_id))
    }

    /// Set a wire column to a cell ID.
    pub fn with_cell(self, name: &'static str, cell_id: CellId) -> Result<Self, QueryError> {
        self.with_wire_ref(name, crate::wire::WireRef::Cell(cell_id))
    }

    /// Set a submodule column value (with optional index).
    #[must_use]
    pub fn with_sub(mut self, name: &'static str, idx: Option<u32>) -> Self {
        let id = T::schema()
            .index_of(name)
            .expect("Schema LUT missing sub column");
        self.entry_array.entries[id] = crate::session::ColumnEntry::Sub { id: idx };
        self
    }

    /// Validate that a selector path exists in the schema.
    ///
    /// Returns `true` if the path is valid, `false` otherwise.
    /// Logs warnings for invalid paths to help debug connection issues.
    pub fn validate_selector_path(selector: crate::selector::Selector<'_>) -> bool {
        if selector.is_empty() {
            tracing::warn!(
                "[{}] Connection uses empty selector path",
                std::any::type_name::<T>()
            );
            return false;
        }

        let head = match selector.head() {
            Some(h) => h,
            None => return false,
        };

        let schema = T::schema();

        // Check if the first segment exists in schema
        if schema.index_of(head).is_none() {
            let available: Vec<&str> = schema.columns().iter().map(|c| c.name).collect();

            tracing::warn!(
                "[{}] Connection path '{}' references non-existent field/submodule.\n  \
                 Available columns: [{}]",
                std::any::type_name::<T>(),
                selector.path().join("."),
                available.join(", "),
            );
            return false;
        }

        // If multi-segment, verify it's a submodule
        if selector.len() > 1 {
            let col_idx = T::schema().index_of(head).unwrap();
            let col_def = T::schema().column(col_idx);

            if !col_def.kind.is_sub() {
                tracing::warn!(
                    "[{}] Connection path '{}' tries to traverse into '{}', but it's not a submodule (it's a {:?})",
                    std::any::type_name::<T>(),
                    selector.path().join("."),
                    head,
                    col_def.kind
                );
                return false;
            }
        }

        true
    }

    pub fn render(&self, store: &Store, driver: &Driver, key: &DriverKey) -> String
    where
        T: Pattern + svql_query::traits::Component + 'static,
    {
        T::render_row(self, store, driver, key)
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
        for (idx, col_def) in T::schema().columns().iter().enumerate() {
            let entry = self.entry_array.entries.get(idx);

            let value_str = match entry {
                Some(crate::session::ColumnEntry::Wire {
                    value: Some(wire_ref),
                }) => match wire_ref {
                    crate::wire::WireRef::Cell(id) => format!("cell({id})"),
                    crate::wire::WireRef::PrimaryPort(name) => format!("port({})", name),
                    crate::wire::WireRef::Constant(val) => format!("const({})", val),
                },
                Some(crate::session::ColumnEntry::Wire { value: None }) => "wire=NULL".to_string(),
                Some(crate::session::ColumnEntry::Sub { id: Some(id) }) => {
                    format!("ref({id})")
                }
                Some(crate::session::ColumnEntry::Sub { id: None }) => "ref=NULL".to_string(),
                Some(crate::session::ColumnEntry::Metadata { id: Some(id) }) => {
                    format!("meta({id})")
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
