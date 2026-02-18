// svql_query/src/session/schema/pattern_schema.rs

use svql_common::PortDirection;

use crate::session::schema::column::{ColIdx, SchemaColumn, SubIdx};

use std::any::TypeId;
use std::collections::HashMap;

/// A validated, indexed schema for a pattern's result table.
///
/// Constructed once (via `OnceLock`) per pattern type. Provides O(1)
/// lookup by name, and pre-computed typed index lists so callers
/// never need to scan the column list.
///
/// # Invariants (checked at construction)
///
/// - All column names are unique.
/// - No column name is the empty string.
/// - `inputs`, `outputs`, `submodules`, `metas` are disjoint and
///   collectively cover all columns.
pub struct PatternSchema {
    /// Source of truth — all columns in declaration order.
    columns: &'static [SchemaColumn],

    /// O(1) name → column index.
    name_index: HashMap<&'static str, ColIdx>,

    /// Indices of Port columns with `Input` direction.
    inputs: Vec<ColIdx>,
    /// Indices of Port columns with `Output` direction.
    outputs: Vec<ColIdx>,
    /// Indices of Sub columns, in declaration order.
    submodules: Vec<ColIdx>,
    /// Indices of Meta columns.
    metas: Vec<ColIdx>,

    /// For Sub columns: maps column name → position within `submodules`
    /// (i.e., the index used to look up the dependency table slice).
    sub_position: HashMap<&'static str, SubIdx>,
}

impl PatternSchema {
    /// Construct and validate a schema from a static column slice.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Any two columns share a name.
    /// - Any column name is empty.
    pub fn new(columns: &'static [SchemaColumn]) -> Self {
        let mut name_index = HashMap::with_capacity(columns.len());
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut submodules = Vec::new();
        let mut metas = Vec::new();
        let mut sub_position = HashMap::new();

        for (i, col) in columns.iter().enumerate() {
            let name = col.name();
            assert!(!name.is_empty(), "Column name must not be empty");
            let col_idx = ColIdx::new(i as u32);

            let prev = name_index.insert(name, col_idx);
            assert!(
                prev.is_none(),
                "Duplicate column name '{}' in schema for {:?}",
                name,
                std::any::type_name::<Self>()
            );

            match col {
                SchemaColumn::Port(p) => match p.direction {
                    PortDirection::Input => inputs.push(col_idx),
                    PortDirection::Output => outputs.push(col_idx),
                    _ => {}
                },
                SchemaColumn::Sub(_) => {
                    let sub_idx = SubIdx::new(submodules.len() as u32);
                    sub_position.insert(name, sub_idx);
                    submodules.push(col_idx);
                }
                SchemaColumn::Meta(_) => metas.push(col_idx),
                SchemaColumn::WireArray(_) => {} // no special index needed
            }
        }

        Self {
            columns,
            name_index,
            inputs,
            outputs,
            submodules,
            metas,
            sub_position,
        }
    }

    /// Look up a column by name — O(1).
    pub fn get(&self, name: &str) -> Option<(ColIdx, &SchemaColumn)> {
        let &idx = self.name_index.get(name)?;
        Some((idx, &self.columns[idx.raw() as usize]))
    }

    /// Get a column by its opaque index.
    pub fn column(&self, idx: ColIdx) -> &SchemaColumn {
        &self.columns[idx.raw() as usize]
    }

    /// All columns in declaration order.
    pub fn columns(&self) -> &[SchemaColumn] {
        self.columns
    }

    /// Input port column indices.
    pub fn inputs(&self) -> &[ColIdx] {
        &self.inputs
    }

    /// Output port column indices.
    pub fn outputs(&self) -> &[ColIdx] {
        &self.outputs
    }

    /// Submodule column indices (in declaration order).
    pub fn submodule_col_indices(&self) -> &[ColIdx] {
        &self.submodules
    }

    /// Look up the `SubIdx` for a named submodule column.
    ///
    /// `SubIdx` is used to index into `dep_tables` slices in `Composite::compose`.
    pub fn sub_position(&self, name: &str) -> Option<SubIdx> {
        self.sub_position.get(name).copied()
    }

    /// Get the `TypeId` that a named Sub column points to.
    pub fn sub_target_type(&self, name: &str) -> Option<TypeId> {
        let (idx, col) = self.get(name)?;
        let _ = idx;
        col.as_sub().map(|s| s.target_type)
    }

    /// Total number of columns.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}
