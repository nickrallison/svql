//! Typed DataFrame wrapper for pattern results.
//!
//! `Table<T>` wraps a Polars DataFrame while providing type-safe
//! access to rows as `Row<T>` and references as `Ref<T>`.

use std::any::TypeId;
use std::marker::PhantomData;

use polars::prelude::*;

use crate::session::{EntryArray, Row};
use crate::traits::Pattern;

use super::cell_id::CellId;
use super::error::QueryError;
use super::ref_type::Ref;
// use super::row::Row;
use super::schema::{ColumnDef, ColumnKind};

/// A typed wrapper around a DataFrame storing pattern match results.
///
/// Each row in the DataFrame represents one match of pattern `T`.
/// The schema is defined by `T::COLUMNS` on the `Pattern` trait.
///
/// # Column Types
///
/// - **Wire columns** (`ColumnKind::Wire`): Store as `i64` (CellId::raw())
/// - **Sub columns** (`ColumnKind::Sub`): Store as `u32` (row index or -1 for NULL)
/// - **Metadata columns**: Various types (depth as u32, etc.)
/// - **path**: Always present as `Utf8` column
///
/// # Usage
///
/// ```ignore
/// let table: &Table<MyPattern<Search>> = store.get()?;
/// for row in table.rows() {
///     let wire = row.wire("clk")?;
///     let sub: Ref<Dep<Search>> = row.sub("dep")?;
/// }
/// ```
pub struct Table<T> {
    /// The underlying Polars DataFrame.
    df: DataFrame,
    // /// Column schema for this pattern type.
    // columns: &'static [ColumnDef],
    // /// Mapping from submodule column names to their target TypeId.
    // /// Used for runtime type checking when accessing sub columns.
    // sub_types: HashMap<&'static str, TypeId>,
    /// Type marker.
    _marker: PhantomData<T>,
}

impl<T> Table<T>
where
    T: Pattern,
{
    /// Create a table from a pre-collected set of row matches.
    ///
    /// This is efficient as it performs a single allocation per column.
    pub fn new(rows: Vec<EntryArray>) -> Result<Self, QueryError>
    where
        T: Pattern,
    {
        if rows.is_empty() {
            let cols: Vec<Column> = T::SCHEMA
                .iter()
                .map(|col_def| col_def.into_polars_column())
                .collect();
            let df = DataFrame::new(cols)?;
            return Ok(Self {
                df,
                _marker: PhantomData,
            });
        }

        let mut columns = Vec::with_capacity(T::SCHEMA_SIZE + 1);

        // 1. Handle structural columns from the RowMatch array
        for i in 0..T::SCHEMA_SIZE {
            let col_def = &T::SCHEMA[i];
            let series_data: Vec<u64> = rows.iter().map(|r| r.entries[i].as_u64()).collect();
            columns.push(Column::new(
                PlSmallStr::from_static(col_def.name),
                series_data,
            ));
        }

        let df = DataFrame::new(columns)?;
        Ok(Self {
            df,
            _marker: PhantomData,
        })
    }

    /// Deduplicate rows in the table.
    pub fn deduplicate(&self) -> Result<Self, QueryError> {
        let subset: Vec<String> = T::SCHEMA.iter().map(|c| c.name.to_string()).collect();

        let df = self.df.clone().unique::<Vec<String>, String>(
            Some(&subset),
            UniqueKeepStrategy::First,
            None,
        )?;

        Ok(Self {
            df,
            _marker: PhantomData,
        })
    }

    /// Get the number of rows (matches) in this table.
    #[inline]
    pub fn len(&self) -> usize {
        self.df.height()
    }

    /// Check if the table is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.df.height() == 0
    }

    /// Get a reference to the underlying DataFrame for bulk operations.
    #[inline]
    pub fn df(&self) -> &DataFrame {
        &self.df
    }

    /// Get a mutable reference to the underlying DataFrame.
    #[inline]
    pub fn df_mut(&mut self) -> &mut DataFrame {
        &mut self.df
    }

    /// Get a single row by index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn row(&self, idx: u32) -> Option<Row<T>> {
        let idx_usize = idx as usize;
        if idx_usize >= self.df.height() {
            return None;
        }

        // Extract path
        let path = self
            .df
            .column("path")
            .ok()
            .and_then(|s| s.str().ok())
            .and_then(|ca| ca.get(idx_usize))
            .unwrap_or("")
            .to_string();

        let mut row = Row::new(idx, path);

        // Extract wire and sub columns
        for col in T::columns() {
            match col.kind {
                ColumnKind::Cell => {
                    if let Ok(series) = self.df.column(col.name)
                        && let Ok(ca) = series.i64()
                    {
                        let cell_id = ca.get(idx_usize).map(|raw| CellId::from_raw(raw as u64));
                        row.wires.insert(col.name, cell_id);
                    }
                }
                ColumnKind::Sub(_) => {
                    if let Ok(series) = self.df.column(col.name)
                        && let Ok(ca) = series.u32()
                    {
                        let sub_idx = ca.get(idx_usize).unwrap_or(u32::MAX);
                        row.subs.insert(col.name, sub_idx);
                    }
                }
                ColumnKind::Metadata => {
                    // Handle depth specially
                    if col.name == "depth"
                        && let Ok(series) = self.df.column("depth")
                        && let Ok(ca) = series.u32()
                    {
                        row.depth = ca.get(idx_usize);
                    }
                }
            }
        }

        Some(row)
    }

    /// Iterate over all rows in the table.
    pub fn rows(&self) -> impl Iterator<Item = Row<T>> + '_ {
        (0..self.len() as u32).filter_map(|idx| self.row(idx))
    }

    /// Iterate over references to all rows.
    pub fn refs(&self) -> impl Iterator<Item = Ref<T>> {
        (0..self.len() as u32).map(Ref::new)
    }

    /// Get the TypeId for a submodule column.
    pub fn sub_type(&self, name: &str) -> Option<TypeId> {
        self.sub_types.get(name).copied()
    }
}

impl<T> std::fmt::Debug for Table<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table")
            .field("len", &self.len())
            .field("columns", &self.columns.len())
            .finish()
    }
}

impl<T> std::fmt::Display for Table<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = std::any::type_name::<T>();
        writeln!(
            f,
            "\n╔══════════════════════════════════════════════════════════════════════════════"
        )?;
        writeln!(f, "║ Table: {}  ", type_name)?;
        writeln!(f, "║ Rows: {}", self.len())?;
        writeln!(
            f,
            "╚══════════════════════════════════════════════════════════════════════════════"
        )?;

        if self.is_empty() {
            writeln!(f, "(empty table)")?;
        } else {
            // Use Polars' built-in DataFrame display which handles formatting beautifully
            write!(f, "{}", self.df)?;
        }

        Ok(())
    }
}

/// Type-erased table trait for storing in `Store`.
pub trait AnyTable: Send + Sync + std::fmt::Display + 'static {
    /// Downcast to concrete type.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get the number of rows.
    fn len(&self) -> usize;

    /// Check if empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the type name of the table.
    fn type_name(&self) -> &str;
}

impl<T: Send + Sync + 'static> AnyTable for Table<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn len(&self) -> usize {
        self.df.height()
    }

    fn type_name(&self) -> &str {
        std::any::type_name::<T>()
    }
}

// /// Builder for constructing a Table from rows.
// pub struct TableBuilder<T> {
//     /// Column schema.
//     columns: &'static [ColumnDef],
//     /// Accumulated paths.
//     paths: Vec<String>,
//     /// Accumulated wire values: column_name → values per row.
//     wires: HashMap<&'static str, Vec<Option<i64>>>,
//     /// Accumulated sub values: column_name → values per row.
//     subs: HashMap<&'static str, Vec<Option<u32>>>,
//     /// Accumulated depth values (for tree types).
//     depths: Vec<Option<u32>>,
//     /// Type marker.
//     _marker: PhantomData<T>,
// }

// impl<T> TableBuilder<T> {
//     /// Create a new builder with the given schema.
//     pub fn new(columns: &'static [ColumnDef]) -> Self {
//         let mut wires = HashMap::new();
//         let mut subs = HashMap::new();

//         for col in columns {
//             match col.kind {
//                 ColumnKind::Wire => {
//                     wires.insert(col.name, Vec::new());
//                 }
//                 ColumnKind::Sub(_) => {
//                     subs.insert(col.name, Vec::new());
//                 }
//                 ColumnKind::Metadata => {} // Handle specially
//             }
//         }

//         Self {
//             columns,
//             paths: Vec::new(),
//             wires,
//             subs,
//             depths: Vec::new(),
//             _marker: PhantomData,
//         }
//     }

//     /// Add a row to the builder.
//     pub fn push(&mut self, row: Row<T>) {
//         self.paths.push(row.path);

//         for col in self.columns {
//             match col.kind {
//                 ColumnKind::Wire => {
//                     let val = row
//                         .wires
//                         .get(col.name)
//                         .copied()
//                         .flatten()
//                         .map(|c| c.raw() as i64);
//                     if let Some(vec) = self.wires.get_mut(col.name) {
//                         vec.push(val);
//                     }
//                 }
//                 ColumnKind::Sub(_) => {
//                     let val = row.subs.get(col.name).copied().filter(|&v| v != u32::MAX);
//                     if let Some(vec) = self.subs.get_mut(col.name) {
//                         vec.push(val);
//                     }
//                 }
//                 ColumnKind::Metadata if col.name == "depth" => {
//                     self.depths.push(row.depth);
//                 }
//                 _ => {}
//             }
//         }
//     }

//     /// Get the current number of rows.
//     pub fn len(&self) -> usize {
//         self.paths.len()
//     }

//     /// Check if empty.
//     pub fn is_empty(&self) -> bool {
//         self.paths.is_empty()
//     }

//     /// Build the final Table.
//     pub fn build(self) -> Result<Table<T>, QueryError> {
//         let mut col_vec: Vec<Column> = Vec::with_capacity(self.columns.len() + 2);

//         // Path column
//         col_vec.push(Column::new(PlSmallStr::from_static("path"), self.paths));

//         // Data columns
//         for col in self.columns {
//             match col.kind {
//                 ColumnKind::Wire => {
//                     if let Some(values) = self.wires.get(col.name) {
//                         col_vec.push(Column::new(
//                             PlSmallStr::from_static(col.name),
//                             values.clone(),
//                         ));
//                     }
//                 }
//                 ColumnKind::Sub(_) => {
//                     if let Some(values) = self.subs.get(col.name) {
//                         col_vec.push(Column::new(
//                             PlSmallStr::from_static(col.name),
//                             values.clone(),
//                         ));
//                     }
//                 }
//                 ColumnKind::Metadata if col.name == "depth" => {
//                     col_vec.push(Column::new(
//                         PlSmallStr::from_static("depth"),
//                         self.depths.clone(),
//                     ));
//                 }
//                 _ => {}
//             }
//         }

//         let df = DataFrame::new(col_vec)?;
//         Ok(Table::new(df, self.columns))
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     struct TestPattern;

//     static TEST_COLUMNS: &[ColumnDef] = &[ColumnDef::wire("clk"), ColumnDef::wire_nullable("rst")];

//     #[test]
//     fn test_empty_table() {
//         let table: Table<TestPattern> = Table::empty(TEST_COLUMNS).unwrap();
//         assert!(table.is_empty());
//         assert_eq!(table.len(), 0);
//     }

//     #[test]
//     fn test_table_builder() {
//         let mut builder: TableBuilder<TestPattern> = TableBuilder::new(TEST_COLUMNS);

//         let row1 = Row::<TestPattern>::new(0, "top.a".to_string())
//             .with_wire("clk", Some(CellId::new(100)))
//             .with_wire("rst", Some(CellId::new(200)));

//         let row2 = Row::<TestPattern>::new(1, "top.b".to_string())
//             .with_wire("clk", Some(CellId::new(101)))
//             .with_wire("rst", None);

//         builder.push(row1);
//         builder.push(row2);

//         let table = builder.build().unwrap();
//         assert_eq!(table.len(), 2);

//         let r0 = table.row(0).unwrap();
//         assert_eq!(r0.path(), "top.a");
//         assert_eq!(r0.wire("clk"), Some(CellId::new(100)));

//         let r1 = table.row(1).unwrap();
//         assert_eq!(r1.path(), "top.b");
//         assert_eq!(r1.wire("rst"), None);
//     }

//     #[test]
//     fn test_table_iteration() {
//         let mut builder: TableBuilder<TestPattern> = TableBuilder::new(TEST_COLUMNS);
//         for i in 0..5 {
//             builder.push(
//                 Row::<TestPattern>::new(i, format!("path_{}", i))
//                     .with_wire("clk", Some(CellId::new(i))),
//             );
//         }

//         let table = builder.build().unwrap();
//         let paths: Vec<_> = table.rows().map(|r| r.path().to_string()).collect();
//         assert_eq!(
//             paths,
//             vec!["path_0", "path_1", "path_2", "path_3", "path_4"]
//         );

//         let refs: Vec<_> = table.refs().collect();
//         assert_eq!(refs.len(), 5);
//         assert_eq!(refs[2].index(), 2);
//     }
// }
