//! Owned row snapshot from a pattern's result table.

use std::fmt::Display;
use std::marker::PhantomData;

use crate::prelude::*;
use crate::session::storage::meta_value::MetaValue;
use crate::session::storage::row_index::RowIndex;

/// An owned snapshot of a single row from a `Table<T>`.
#[derive(Debug, Clone)]
pub struct Row<T>
where
    T: Pattern + svql_query::traits::Component,
{
    /// Row index in the source table — opaque outside this module.
    pub(super) idx: RowIndex,
    /// The column data for this row.
    pub(crate) entry_array: EntryArray,
    /// Type marker.
    pub(crate) _marker: PhantomData<T>,
}

impl<T> Row<T>
where
    T: Pattern + svql_query::traits::Component,
{
    /// Create a new row. Only called by `Table` internals.
    #[allow(dead_code)]
    #[must_use]
    pub(super) const fn new(idx: RowIndex) -> Self {
        Self {
            idx,
            entry_array: EntryArray::empty(),
            _marker: PhantomData,
        }
    }

    /// Create a row from its constituent parts.
    pub(crate) const fn from_parts(idx: RowIndex, entry_array: EntryArray) -> Self {
        Self {
            idx,
            entry_array,
            _marker: PhantomData,
        }
    }

    /// Get a typed reference to this row.
    ///
    /// The returned `Ref<T>` can be stored and later passed back to
    /// `Table::row()` to retrieve this row again.
    #[inline]
    #[must_use]
    pub const fn as_ref(&self) -> Ref<T> {
        Ref::new(self.idx)
    }

    /// Accesses the underlying entry array for this row.
    #[inline]
    #[must_use]
    pub const fn entry_array(&self) -> &EntryArray {
        &self.entry_array
    }

    /// Get a wire by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    /// Returns a borrowed reference to avoid cloning at the storage layer.
    #[inline]
    #[must_use]
    pub fn wire(&self, name: &str) -> Option<&Wire> {
        let idx = T::schema().index_of(name)?;
        self.entry_array.entries[idx].as_wire()
    }

    /// Get a port (wire + direction) by column name.
    #[inline]
    #[must_use]
    pub fn port(&self, name: &str) -> Option<Port> {
        let idx = T::schema().index_of(name)?;
        let col_def = T::schema().column(idx);
        let wire = self.entry_array.entries[idx].as_wire()?.clone();
        Some(Port::new(wire, col_def.direction))
    }

    /// Get a bundle of wires by column name.
    ///
    /// Returns `None` if the column doesn't exist or is not a `WireArray`.
    #[inline]
    #[must_use]
    pub fn wire_bundle(&self, name: &str) -> Option<&[Wire]> {
        let idx = T::schema().index_of(name)?;
        self.entry_array.entries[idx].as_wire_array()
    }

    /// Get metadata by column name.
    ///
    /// Returns `None` if the column doesn't exist or is not metadata.
    #[inline]
    #[must_use]
    pub fn meta(&self, name: &str) -> Option<&MetaValue> {
        let idx = T::schema().index_of(name)?;
        self.entry_array.entries[idx].as_meta()
    }

    /// Get a typed submodule reference by column name.
    ///
    /// Returns `None` if:
    /// - The column doesn't exist in the schema
    /// - The column's `TypeId` doesn't match `S`
    /// - The entry is `Null` (optional submodule not matched)
    #[must_use]
    pub fn sub<S: 'static>(&self, name: &str) -> Option<Ref<S>> {
        use std::any::TypeId;
        let idx = T::schema().index_of(name)?;
        let col_def = T::schema().column(idx);

        // Runtime type check: column must point to table of type S.
        match &col_def.kind {
            ColumnKind::Sub(tid) if *tid == TypeId::of::<S>() => {}
            _ => return None,
        }

        self.entry_array.entries[idx].as_row_index().map(Ref::new)
    }

    /// Get a type-erased submodule row index.
    ///
    /// Used internally for display and report generation where the exact
    /// submodule type isn't known at compile time.
    /// Only available within the `session` module.
    pub(crate) fn sub_raw(&self, name: &str) -> Option<u32> {
        let idx = T::schema().index_of(name)?;
        self.entry_array.entries[idx]
            .as_row_index()
            .map(|ri| ri.raw())
    }

    // ── Path resolution ──────────────────────────────────────────────────────

    /// Resolve a selector path to a wire.
    ///
    /// - Single segment `["y"]` → looks up `y` directly.
    /// - Multi-segment `["sub", "y"]` → traverses into submodule `sub`, then looks up `y`.
    #[must_use]
    pub fn resolve(
        &self,
        selector: Selector<'_>,
        ctx: &crate::session::ExecutionContext,
    ) -> Option<Wire> {
        if selector.is_empty() {
            return None;
        }

        if selector.len() == 1 {
            return self.wire(selector.head()?).cloned();
        }

        let head = selector.head()?;
        let idx = T::schema().index_of(head)?;
        let col_def = T::schema().column(idx);

        let (row_idx, sub_type_id) = match &col_def.kind {
            ColumnKind::Sub(tid) => {
                let ri = self.entry_array.entries[idx].as_row_index()?;
                (ri.raw() as usize, *tid)
            }
            _ => return None,
        };

        let sub_table = ctx.get_any_table(sub_type_id)?;
        sub_table.resolve_path(row_idx, selector.tail(), ctx)
    }

    /// Resolve a selector path to a wire bundle.
    #[must_use]
    pub fn resolve_bundle(
        &self,
        selector: Selector<'_>,
        ctx: &crate::session::ExecutionContext,
    ) -> Option<Vec<Wire>> {
        if selector.is_empty() {
            return None;
        }

        if selector.len() == 1 {
            return self.wire_bundle(selector.head()?).map(ToOwned::to_owned);
        }

        let head = selector.head()?;
        let idx = T::schema().index_of(head)?;
        let col_def = T::schema().column(idx);

        let (row_idx, sub_type_id) = match &col_def.kind {
            ColumnKind::Sub(tid) => {
                let ri = self.entry_array.entries[idx].as_row_index()?;
                (ri.raw() as usize, *tid)
            }
            _ => return None,
        };

        let sub_table = ctx.get_any_table(sub_type_id)?;
        sub_table.resolve_bundle_path(row_idx, selector.tail(), ctx)
    }

    // ── Builder methods (used by Table / pattern search code) ────────────────

    /// Set a wire column value.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the column name does not exist in the schema.
    pub fn with_wire(mut self, name: &'static str, wire: Wire) -> Result<Self, QueryError> {
        let id = T::schema()
            .index_of(name)
            .ok_or_else(|| QueryError::SchemaLut(name.to_string()))?;
        self.entry_array.entries[id] = ColumnEntry::wire(wire);
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
        self.entry_array.entries[id] = ColumnEntry::wire_array(wires);
        Ok(self)
    }

    /// Validate that a selector path exists in the schema.
    ///
    /// # Panics
    ///
    /// May panic if the schema index lookup fails unexpectedly.
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

        if selector.len() > 1 {
            let col_idx = T::schema().index_of(head).unwrap();
            let col_def = T::schema().column(col_idx);

            if !col_def.kind.is_sub() {
                tracing::warn!(
                    "[{}] Connection path '{}' tries to traverse into '{}', \
                     but it's not a submodule (it's a {:?})",
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
    pub fn render(
        &self,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
        config: &svql_common::Config,
    ) -> String
    where
        T: Pattern + svql_query::traits::Component + 'static,
    {
        T::render_row(self, store, driver, key, config)
    }
}

impl<T> Display for Row<T>
where
    T: Pattern + svql_query::traits::Component,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap_or("Row");

        writeln!(f, "{}[{}]:", type_name, self.idx)?;

        for (idx, col_def) in T::schema().columns().iter().enumerate() {
            let entry = self.entry_array.entries.get(idx);

            let value_str = match entry {
                Some(ColumnEntry::Wire(wire)) => format!("{}", wire),
                Some(ColumnEntry::WireArray(wires)) => {
                    let strs: Vec<String> = wires.iter().map(|w| format!("{}", w)).collect();
                    format!("[{}]", strs.join(", "))
                }
                Some(ColumnEntry::Sub(idx)) => format!("ref({})", idx),
                Some(ColumnEntry::Meta(m)) => format!("{}", m),
                Some(ColumnEntry::Null) => "NULL".to_string(),
                None => "MISSING".to_string(),
            };

            let nullable_marker = if col_def.nullable { "?" } else { "" };
            writeln!(f, "  {}{}: {}", col_def.name, nullable_marker, value_str)?;
        }

        Ok(())
    }
}
