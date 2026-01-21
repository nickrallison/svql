//! Owned row snapshot from a pattern's result table.
//!
//! `Row<T>` provides an owned copy of a single row's data, avoiding
//! lifetime complexity when iterating or passing rows around.

use std::collections::HashMap;
use std::marker::PhantomData;

use super::cell_id::CellId;
use super::ref_type::Ref;

/// An owned snapshot of a single row from a `Table<T>`.
///
/// This is created by `Table::row()` or during iteration, and holds
/// all the data needed to reconstruct a `T::Match` via `Pattern::rehydrate()`.
///
/// # Field Access
///
/// - `wire("name")` - Get a wire reference (CellId)
/// - `sub::<S>("name")` - Get a submodule reference (Ref<S>)
/// - `path()` - Get the hierarchical path string
/// - `depth()` - Get the tree depth (for RecOr/RecAnd)
/// - `left_child()` / `right_child()` - Get tree children (for RecOr/RecAnd)
#[derive(Debug, Clone)]
pub struct Row<T> {
    /// Row index in the source table.
    pub(crate) idx: u32,
    /// Hierarchical path (e.g., "top.cpu.alu.adder").
    pub(crate) path: String,
    /// Wire columns: name → CellId (None if NULL).
    pub(crate) wires: HashMap<&'static str, Option<CellId>>,
    /// Submodule columns: name → row index in target table.
    /// u32::MAX represents NULL.
    pub(crate) subs: HashMap<&'static str, u32>,
    /// Depth in tree structure (for RecOr/RecAnd).
    pub(crate) depth: Option<u32>,
    /// Type marker.
    pub(crate) _marker: PhantomData<T>,
}

/// Sentinel value for NULL submodule references.
const NULL_REF: u32 = u32::MAX;

impl<T> Row<T> {
    /// Create a new row (typically called by Table).
    pub fn new(idx: u32, path: String) -> Self {
        Self {
            idx,
            path,
            wires: HashMap::new(),
            subs: HashMap::new(),
            depth: None,
            _marker: PhantomData,
        }
    }

    /// Get the row index in the source table.
    #[inline]
    pub fn index(&self) -> u32 {
        self.idx
    }

    /// Get this row as a typed reference.
    #[inline]
    pub fn as_ref(&self) -> Ref<T> {
        Ref::new(self.idx)
    }

    /// Get the hierarchical path.
    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get a wire reference by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    #[inline]
    pub fn wire(&self, name: &str) -> Option<CellId> {
        self.wires.get(name).copied().flatten()
    }

    /// Get a submodule reference by column name.
    ///
    /// Returns `None` if the column doesn't exist or the value is NULL.
    pub fn sub<S>(&self, name: &str) -> Option<Ref<S>> {
        let &idx = self.subs.get(name)?;
        if idx == NULL_REF {
            None
        } else {
            Some(Ref::new(idx))
        }
    }

    /// Get the tree depth (for recursive types like RecOr/RecAnd).
    ///
    /// Returns `None` for non-tree types.
    #[inline]
    pub fn depth(&self) -> Option<u32> {
        self.depth
    }

    /// Get the left child reference (for tree types).
    ///
    /// This is a convenience method that calls `sub::<T>("left_child")`.
    #[inline]
    pub fn left_child(&self) -> Option<Ref<T>> {
        self.sub("left_child")
    }

    /// Get the right child reference (for tree types).
    ///
    /// This is a convenience method that calls `sub::<T>("right_child")`.
    #[inline]
    pub fn right_child(&self) -> Option<Ref<T>> {
        self.sub("right_child")
    }

    // --- Builder methods (used by Table when constructing rows) ---

    /// Set a wire column value.
    pub fn with_wire(mut self, name: &'static str, cell_id: Option<CellId>) -> Self {
        self.wires.insert(name, cell_id);
        self
    }

    /// Set a submodule column value.
    pub fn with_sub(mut self, name: &'static str, idx: Option<u32>) -> Self {
        self.subs.insert(name, idx.unwrap_or(NULL_REF));
        self
    }

    /// Set the depth value.
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = Some(depth);
        self
    }
}

impl<T> Default for Row<T> {
    fn default() -> Self {
        Self::new(0, String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Dummy types for testing
    struct PatternA;
    struct PatternB;

    #[test]
    fn test_row_basic() {
        let row: Row<PatternA> = Row::new(42, "top.cpu".to_string());
        assert_eq!(row.index(), 42);
        assert_eq!(row.path(), "top.cpu");
    }

    #[test]
    fn test_row_wire_access() {
        let row: Row<PatternA> = Row::new(0, "".to_string())
            .with_wire("clk", Some(CellId::new(100)))
            .with_wire("rst", None);

        assert_eq!(row.wire("clk"), Some(CellId::new(100)));
        assert_eq!(row.wire("rst"), None); // NULL
        assert_eq!(row.wire("nonexistent"), None);
    }

    #[test]
    fn test_row_sub_access() {
        let row: Row<PatternA> = Row::new(0, "".to_string())
            .with_sub("dff", Some(10))
            .with_sub("mux", None); // NULL

        let dff_ref: Option<Ref<PatternB>> = row.sub("dff");
        assert!(dff_ref.is_some());
        assert_eq!(dff_ref.unwrap().index(), 10);

        let mux_ref: Option<Ref<PatternB>> = row.sub("mux");
        assert!(mux_ref.is_none()); // NULL

        let missing: Option<Ref<PatternB>> = row.sub("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_row_tree_methods() {
        let row: Row<PatternA> = Row::new(0, "".to_string())
            .with_sub("left_child", Some(1))
            .with_sub("right_child", Some(2))
            .with_depth(3);

        assert_eq!(row.depth(), Some(3));
        assert_eq!(row.left_child().map(|r| r.index()), Some(1));
        assert_eq!(row.right_child().map(|r| r.index()), Some(2));
    }

    #[test]
    fn test_row_as_ref() {
        let row: Row<PatternA> = Row::new(42, "".to_string());
        let r: Ref<PatternA> = row.as_ref();
        assert_eq!(r.index(), 42);
    }
}
