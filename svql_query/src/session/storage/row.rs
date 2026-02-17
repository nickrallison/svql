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
    /// Wire columns: name → `PhysicalCellId` (None if NULL).
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

    /// Accesses the underlying fixed-size array of data for this row.
    #[inline]
    #[must_use]
    pub const fn entry_array(&self) -> &EntryArray {
        &self.entry_array
    }

    /// Get a wire by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    #[inline]
    #[must_use]
    pub fn wire(&self, name: &str) -> Option<Wire> {
        let idx = T::schema().index_of(name)?;
        match &self.entry_array.entries[idx] {
            ColumnEntry::Wire(wire) => Some(wire.clone()),
            _ => None,
        }
    }

    /// Get a port (wire + direction) by column name.
    ///
    /// Looks up the direction from the schema.
    #[inline]
    #[must_use]
    pub fn port(&self, name: &str) -> Option<Port> {
        let idx = T::schema().index_of(name)?;
        let col_def = T::schema().column(idx);

        let wire = match &self.entry_array.entries[idx] {
            ColumnEntry::Wire(w) => w.clone(),
            _ => return None,
        };

        Some(Port::new(wire, col_def.direction))
    }

    /// Get a bundle of wires by column name.
    ///
    /// Returns `None` if the column doesn't exist or is not a WireArray.
    /// Used for set-based connectivity checking with `#[connect_any]`.
    #[inline]
    #[must_use]
    pub fn wire_bundle(&self, name: &str) -> Option<Vec<Wire>> {
        let idx = T::schema().index_of(name)?;

        match &self.entry_array.entries[idx] {
            ColumnEntry::WireArray(wires) => Some(wires.clone()),
            _ => None,
        }
    }

    /// Get a submodule reference by name (type-erased)
    pub fn sub_any(&self, name: &str) -> Option<u32> {
        let idx = T::schema().index_of(name)?;
        match &self.entry_array.entries[idx] {
            ColumnEntry::Sub(slot_idx) => Some(*slot_idx),
            _ => None,
        }
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
        selector: Selector<'_>,
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
            crate::session::ColumnKind::Sub(tid) => match &self.entry_array.entries[idx] {
                ColumnEntry::Sub(slot_idx) => (*slot_idx, *tid),
                _ => return None,
            },
            _ => return None,
        };

        // Get the submodule's table and continue resolution
        let sub_table = ctx.get_any_table(sub_type_id)?;
        sub_table.resolve_path(sub_row_idx as usize, selector.tail(), ctx)
    }

    /// Resolve a path to a wire bundle using a Selector.
    ///
    /// Returns `None` if the path doesn't resolve to a WireArray column.
    /// Used for set-based connectivity checking with `#[connect_any]`.
    #[must_use]
    pub fn resolve_bundle(
        &self,
        selector: Selector<'_>,
        ctx: &crate::session::ExecutionContext,
    ) -> Option<Vec<Wire>> {
        if selector.is_empty() {
            return None;
        }

        // Single segment: direct lookup
        if selector.len() == 1 {
            return self.wire_bundle(selector.head()?);
        }

        // Multi-segment: traverse through submodules
        let head = selector.head()?;
        let idx = T::schema().index_of(head)?;
        let col_def = T::schema().column(idx);

        // Head must be a submodule reference
        let (sub_row_idx, sub_type_id) = match &col_def.kind {
            crate::session::ColumnKind::Sub(tid) => match &self.entry_array.entries[idx] {
                ColumnEntry::Sub(slot_idx) => (*slot_idx, *tid),
                _ => return None,
            },
            _ => return None,
        };

        // Get the submodule's table and continue resolution
        let sub_table = ctx.get_any_table(sub_type_id)?;
        sub_table.resolve_bundle_path(sub_row_idx as usize, selector.tail(), ctx)
    }

    /// Get a submodule reference by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    #[must_use]
    pub fn sub<S>(&self, name: &str) -> Option<Ref<S>> {
        let idx = T::schema().index_of(name)?;

        match &self.entry_array.entries[idx] {
            ColumnEntry::Sub(slot_idx) => Some(Ref::new(*slot_idx)),
            _ => None,
        }
    }

    // --- Builder methods (used by Table when constructing rows) ---

    /// Set a wire column value.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the column name does not exist in the schema.
    pub fn with_wire(
        mut self,
        name: &'static str,
        wire: Wire,
    ) -> Result<Self, QueryError> {
        let id = T::schema()
            .index_of(name)
            .ok_or_else(|| QueryError::SchemaLut(name.to_string()))?;
        self.entry_array.entries[id] = ColumnEntry::Wire(wire);
        Ok(self)
    }

    /// Set a wire column value with a cell ID.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the column name does not exist in the schema.
    pub fn with_cell(
        self,
        name: &'static str,
        cell_id: PhysicalCellId,
    ) -> Result<Self, QueryError> {
        self.with_wire(name, Wire::from(cell_id))
    }

    /// Set a wire array column value.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the column name does not exist in the schema.
    pub fn with_wire_array(
        mut self,
        name: &'static str,
        wires: Vec<Wire>,
    ) -> Result<Self, QueryError> {
        let id = T::schema()
            .index_of(name)
            .ok_or_else(|| QueryError::SchemaLut(name.to_string()))?;
        self.entry_array.entries[id] = ColumnEntry::WireArray(wires);
        Ok(self)
    }

    /// Set a submodule column value (with optional index).
    ///
    /// # Panics
    ///
    /// Panics if the requested column name is missing from the pattern schema.
    #[must_use]
    pub fn with_sub(mut self, name: &'static str, idx: Option<u32>) -> Self {
        let id = T::schema()
            .index_of(name)
            .expect("Schema LUT missing sub column");
        self.entry_array.entries[id] = idx.map_or(ColumnEntry::Null, ColumnEntry::Sub);
        self
    }

    /// Validate that a selector path exists in the schema.
    ///
    /// Returns `true` if the path is valid, `false` otherwise.
    /// Logs warnings for invalid paths to help debug connection issues.
    ///
    /// # Panics
    ///
    /// Panics if the requested column name is missing from the pattern schema.
    pub fn validate_selector_path(selector: Selector<'_>) -> bool {
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

    /// Formats the row as a hierarchical string report.
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
                Some(ColumnEntry::Wire(wire)) => format!("{}", wire),
                Some(ColumnEntry::WireArray(wires)) => {
                    let wire_strs: Vec<String> = wires.iter().map(|w| format!("{}", w)).collect();
                    format!("[{}]", wire_strs.join(", "))
                }
                Some(ColumnEntry::Sub(slot_idx)) => format!("ref({})", slot_idx),
                Some(ColumnEntry::Metadata(id)) => format!("meta({id})"),
                Some(ColumnEntry::Null) => "NULL".to_string(),
                None => "MISSING".to_string(),
            };

            let nullable_marker = if col_def.nullable { "?" } else { "" };
            writeln!(f, "  {}{}: {}", col_def.name, nullable_marker, value_str)?;
        }

        Ok(())
    }
}
