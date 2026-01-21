//! Builder for constructing tree-structured result tables.
//!
//! Trees like `RecOr` and `RecAnd` use arena-style storage where parent rows
//! reference child rows via `left_child` and `right_child` columns pointing
//! to the same table.

use std::collections::HashMap;
use std::marker::PhantomData;

use super::cell_id::CellId;
use super::column::ColumnDef;
use super::error::QueryError;
use super::ref_type::Ref;
use super::row::Row;
use super::table::{Table, TableBuilder};

/// A temporary handle to a row being built in a TreeTableBuilder.
///
/// This is used to build parent-child relationships before the final
/// table is constructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TempRef(u32);

impl TempRef {
    /// Get the index that will be used in the final table.
    pub fn index(&self) -> u32 {
        self.0
    }
}

/// Builder for constructing tree-structured tables.
///
/// Unlike `TableBuilder`, this supports:
/// - Building parent rows that reference child rows via `TempRef`
/// - Automatic depth calculation
/// - Arena-style self-referential storage
///
/// # Example
///
/// ```ignore
/// let mut builder = TreeTableBuilder::<RecOr<Search>>::new(RecOr::COLUMNS);
///
/// // Add leaf nodes first
/// let leaf1 = builder.add_leaf("top.or1", |row| {
///     row.with_wire("a", Some(CellId::new(10)))
///        .with_wire("y", Some(CellId::new(12)))
/// });
///
/// // Add parent that references the leaf
/// let parent = builder.add_node("top.or2", Some(leaf1), None, |row| {
///     row.with_wire("a", Some(CellId::new(20)))
///        .with_wire("y", Some(CellId::new(22)))
/// });
///
/// let table = builder.build()?;
/// ```
pub struct TreeTableBuilder<T> {
    /// Column schema.
    columns: &'static [ColumnDef],
    /// Accumulated rows.
    rows: Vec<TreeRow>,
    /// Type marker.
    _marker: PhantomData<T>,
}

/// Internal row representation during tree building.
struct TreeRow {
    path: String,
    wires: HashMap<&'static str, Option<CellId>>,
    left_child: Option<TempRef>,
    right_child: Option<TempRef>,
    depth: u32,
}

impl<T> TreeTableBuilder<T> {
    /// Create a new tree table builder with the given schema.
    pub fn new(columns: &'static [ColumnDef]) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Add a leaf node (no children).
    ///
    /// Returns a `TempRef` that can be used to reference this row from parent nodes.
    pub fn add_leaf<F>(&mut self, path: impl Into<String>, configure: F) -> TempRef
    where
        F: FnOnce(TreeRowBuilder) -> TreeRowBuilder,
    {
        let idx = self.rows.len() as u32;
        let builder = TreeRowBuilder::new(path.into());
        let builder = configure(builder);
        
        self.rows.push(TreeRow {
            path: builder.path,
            wires: builder.wires,
            left_child: None,
            right_child: None,
            depth: 1,
        });
        
        TempRef(idx)
    }

    /// Add a node with optional left and/or right children.
    ///
    /// Depth is automatically calculated as `1 + max(left_depth, right_depth)`.
    pub fn add_node<F>(
        &mut self,
        path: impl Into<String>,
        left_child: Option<TempRef>,
        right_child: Option<TempRef>,
        configure: F,
    ) -> TempRef
    where
        F: FnOnce(TreeRowBuilder) -> TreeRowBuilder,
    {
        let idx = self.rows.len() as u32;
        let builder = TreeRowBuilder::new(path.into());
        let builder = configure(builder);
        
        // Calculate depth
        let left_depth = left_child
            .map(|r| self.rows.get(r.0 as usize).map(|row| row.depth).unwrap_or(0))
            .unwrap_or(0);
        let right_depth = right_child
            .map(|r| self.rows.get(r.0 as usize).map(|row| row.depth).unwrap_or(0))
            .unwrap_or(0);
        let depth = 1 + left_depth.max(right_depth);
        
        self.rows.push(TreeRow {
            path: builder.path,
            wires: builder.wires,
            left_child,
            right_child,
            depth,
        });
        
        TempRef(idx)
    }

    /// Add a unary node (single child, stored as left_child).
    pub fn add_unary<F>(
        &mut self,
        path: impl Into<String>,
        child: TempRef,
        configure: F,
    ) -> TempRef
    where
        F: FnOnce(TreeRowBuilder) -> TreeRowBuilder,
    {
        self.add_node(path, Some(child), None, configure)
    }

    /// Get the current number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get the depth of a row by its TempRef.
    pub fn depth(&self, r: TempRef) -> Option<u32> {
        self.rows.get(r.0 as usize).map(|row| row.depth)
    }

    /// Build the final Table.
    pub fn build(self) -> Result<Table<T>, QueryError>
    where
        T: Send + Sync + 'static,
    {
        let mut table_builder = TableBuilder::<T>::new(self.columns);
        
        for tree_row in self.rows {
            let mut row = Row::<T>::new(0, tree_row.path);
            
            // Copy wire values
            for (name, cell_id) in tree_row.wires {
                row.wires.insert(name, cell_id);
            }
            
            // Set child references
            if let Some(left) = tree_row.left_child {
                row.subs.insert("left_child", left.0);
            } else {
                row.subs.insert("left_child", u32::MAX); // NULL
            }
            
            if let Some(right) = tree_row.right_child {
                row.subs.insert("right_child", right.0);
            } else {
                row.subs.insert("right_child", u32::MAX); // NULL
            }
            
            // Set depth
            row.depth = Some(tree_row.depth);
            
            table_builder.push(row);
        }
        
        table_builder.build()
    }
}

/// Builder for configuring a single tree row's wire values.
pub struct TreeRowBuilder {
    path: String,
    wires: HashMap<&'static str, Option<CellId>>,
}

impl TreeRowBuilder {
    fn new(path: String) -> Self {
        Self {
            path,
            wires: HashMap::new(),
        }
    }

    /// Set a wire column value.
    pub fn with_wire(mut self, name: &'static str, cell_id: Option<CellId>) -> Self {
        self.wires.insert(name, cell_id);
        self
    }
}

/// Extension methods for Row<T> when T is a tree type.
pub trait TreeRowExt {
    /// Check if this is a leaf node (no children).
    fn is_leaf(&self) -> bool;
    
    /// Check if this is a unary node (only left child).
    fn is_unary(&self) -> bool;
    
    /// Check if this is a binary node (both children).
    fn is_binary(&self) -> bool;
}

impl<T> TreeRowExt for Row<T> {
    fn is_leaf(&self) -> bool {
        self.left_child().is_none() && self.right_child().is_none()
    }
    
    fn is_unary(&self) -> bool {
        self.left_child().is_some() && self.right_child().is_none()
    }
    
    fn is_binary(&self) -> bool {
        self.left_child().is_some() && self.right_child().is_some()
    }
}

/// Helper to iterate over a tree in pre-order (parent before children).
pub struct TreePreOrderIter<'a, T> {
    table: &'a Table<T>,
    stack: Vec<Ref<T>>,
}

impl<'a, T: 'static> TreePreOrderIter<'a, T> {
    /// Create a new pre-order iterator starting from the given root.
    pub fn new(table: &'a Table<T>, root: Ref<T>) -> Self {
        Self {
            table,
            stack: vec![root],
        }
    }
}

impl<'a, T: 'static> Iterator for TreePreOrderIter<'a, T> {
    type Item = Row<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.stack.pop()?;
        let row = self.table.row(r.index())?;
        
        // Push children in reverse order so left is processed first
        if let Some(right) = row.right_child() {
            self.stack.push(right);
        }
        if let Some(left) = row.left_child() {
            self.stack.push(left);
        }
        
        Some(row)
    }
}

/// Helper to iterate over a tree in post-order (children before parent).
pub struct TreePostOrderIter<'a, T> {
    table: &'a Table<T>,
    /// Stack of (ref, visited_children) pairs
    stack: Vec<(Ref<T>, bool)>,
}

impl<'a, T: 'static> TreePostOrderIter<'a, T> {
    /// Create a new post-order iterator starting from the given root.
    pub fn new(table: &'a Table<T>, root: Ref<T>) -> Self {
        Self {
            table,
            stack: vec![(root, false)],
        }
    }
}

impl<'a, T: 'static> Iterator for TreePostOrderIter<'a, T> {
    type Item = Row<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((r, visited)) = self.stack.pop() {
            let row = self.table.row(r.index())?;
            
            if visited {
                return Some(row);
            }
            
            // Re-push this node, marked as visited
            self.stack.push((r, true));
            
            // Push children (right first so left is processed first)
            if let Some(right) = row.right_child() {
                self.stack.push((right, false));
            }
            if let Some(left) = row.left_child() {
                self.stack.push((left, false));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTree;

    fn tree_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::wire("a"),
            ColumnDef::wire("y"),
            // Tree-specific columns required by TableBuilder
            ColumnDef::sub_nullable::<TestTree>("left_child"),
            ColumnDef::sub_nullable::<TestTree>("right_child"),
            ColumnDef::metadata("depth"),
        ]
    }
    
    // Use Box::leak to create static slices for testing
    fn static_tree_columns() -> &'static [ColumnDef] {
        Box::leak(tree_columns().into_boxed_slice())
    }

    #[test]
    fn test_tree_builder_leaf() {
        let mut builder = TreeTableBuilder::<TestTree>::new(static_tree_columns());
        
        let leaf = builder.add_leaf("top.or1", |row| {
            row.with_wire("a", Some(CellId::new(10)))
               .with_wire("y", Some(CellId::new(12)))
        });
        
        assert_eq!(builder.len(), 1);
        assert_eq!(builder.depth(leaf), Some(1));
    }

    #[test]
    fn test_tree_builder_parent_child() {
        let mut builder = TreeTableBuilder::<TestTree>::new(static_tree_columns());
        
        let leaf = builder.add_leaf("top.or1", |row| {
            row.with_wire("a", Some(CellId::new(10)))
        });
        
        let parent = builder.add_unary("top.or2", leaf, |row| {
            row.with_wire("a", Some(CellId::new(20)))
        });
        
        assert_eq!(builder.len(), 2);
        assert_eq!(builder.depth(leaf), Some(1));
        assert_eq!(builder.depth(parent), Some(2));
    }

    #[test]
    fn test_tree_builder_binary() {
        let mut builder = TreeTableBuilder::<TestTree>::new(static_tree_columns());
        
        let left = builder.add_leaf("left", |r| r);
        let right = builder.add_leaf("right", |r| r);
        let root = builder.add_node("root", Some(left), Some(right), |r| r);
        
        assert_eq!(builder.depth(root), Some(2));
    }

    #[test]
    fn test_tree_builder_build() {
        let mut builder = TreeTableBuilder::<TestTree>::new(static_tree_columns());
        
        let leaf = builder.add_leaf("leaf", |row| {
            row.with_wire("a", Some(CellId::new(100)))
        });
        let _parent = builder.add_unary("parent", leaf, |row| {
            row.with_wire("a", Some(CellId::new(200)))
        });
        
        let table = builder.build().unwrap();
        assert_eq!(table.len(), 2);
        
        let row0 = table.row(0).unwrap();
        assert_eq!(row0.path(), "leaf");
        assert_eq!(row0.depth(), Some(1));
        assert!(row0.left_child().is_none());
        
        let row1 = table.row(1).unwrap();
        assert_eq!(row1.path(), "parent");
        assert_eq!(row1.depth(), Some(2));
        assert!(row1.left_child().is_some());
        assert_eq!(row1.left_child().unwrap().index(), 0);
    }
}
