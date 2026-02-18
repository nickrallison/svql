use std::any::TypeId;

use svql_common::Wire;

use crate::session::{
    ColumnEntry, MetaValue, RowIndex,
    schema::{error::SchemaError, pattern_schema::PatternSchema},
};

/// An array of column entries representing a single row.
pub struct EntryArray {
    /// The entries in this row.
    #[allow(dead_code)]
    entries: Vec<ColumnEntry>,
}

/// A builder for constructing entry arrays with schema validation.
pub struct EntryArrayBuilder<'schema> {
    /// The schema used for validation.
    schema: &'schema PatternSchema,
    /// The entries being built.
    entries: Vec<ColumnEntry>,
}

impl<'schema> EntryArrayBuilder<'schema> {
    /// Create a new builder for the given schema.
    pub fn new(schema: &'schema PatternSchema) -> Self {
        Self {
            schema,
            entries: vec![ColumnEntry::Null; schema.len()],
        }
    }

    /// Set a wire value for a column.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The column name is not found in the schema
    /// - The column is not a port column
    pub fn set_wire(mut self, name: &str, wire: Wire) -> Result<Self, SchemaError> {
        let (idx, col) = self
            .schema
            .get(name)
            .ok_or_else(|| SchemaError::UnknownColumn(name.to_string()))?;
        if !col.is_port() {
            return Err(SchemaError::WrongKind {
                name: name.to_string(),
                expected: "Port",
                actual: col.kind_name(),
            });
        }
        self.entries[idx.raw() as usize] = ColumnEntry::Wire(wire);
        Ok(self)
    }

    /// Set a wire array value for a column.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The column name is not found in the schema
    /// - The column is not a wire array column
    pub fn set_wire_array(mut self, name: &str, wires: Vec<Wire>) -> Result<Self, SchemaError> {
        let (idx, col) = self
            .schema
            .get(name)
            .ok_or_else(|| SchemaError::UnknownColumn(name.to_string()))?;
        if col.as_wire_array().is_none() {
            return Err(SchemaError::WrongKind {
                name: name.to_string(),
                expected: "WireArray",
                actual: col.kind_name(),
            });
        }
        self.entries[idx.raw() as usize] = ColumnEntry::WireArray(wires);
        Ok(self)
    }

    /// Set a sub-entry (row reference) for a column.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The column name is not found in the schema
    /// - The column is not a sub column
    /// - The target type does not match the expected type
    pub fn set_sub<S: 'static>(mut self, name: &str, row: RowIndex) -> Result<Self, SchemaError> {
        let (idx, col) = self
            .schema
            .get(name)
            .ok_or_else(|| SchemaError::UnknownColumn(name.to_string()))?;
        let sub_col = col.as_sub().ok_or_else(|| SchemaError::WrongKind {
            name: name.to_string(),
            expected: "Sub",
            actual: col.kind_name(),
        })?;
        if sub_col.target_type != TypeId::of::<S>() {
            return Err(SchemaError::SubTypeMismatch {
                column: name.to_string(),
                expected: sub_col.target_type_name,
                actual: std::any::type_name::<S>(),
            });
        }
        self.entries[idx.raw() as usize] = ColumnEntry::Sub(row);
        Ok(self)
    }

    /// Set a metadata value for a column.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The column name is not found in the schema
    /// - The column is not a metadata column
    pub fn set_meta(mut self, name: &str, value: MetaValue) -> Result<Self, SchemaError> {
        let (idx, col) = self
            .schema
            .get(name)
            .ok_or_else(|| SchemaError::UnknownColumn(name.to_string()))?;
        if !col.is_meta() {
            return Err(SchemaError::WrongKind {
                name: name.to_string(),
                expected: "Meta",
                actual: col.kind_name(),
            });
        }
        self.entries[idx.raw() as usize] = ColumnEntry::Meta(value);
        Ok(self)
    }

    /// Finish building the entry array, validating that all required columns are set.
    ///
    /// # Errors
    ///
    /// Returns an error if any required (non-nullable) column was not set.
    pub fn finish(self) -> Result<EntryArray, SchemaError> {
        for (i, (entry, col)) in self
            .entries
            .iter()
            .zip(self.schema.columns().iter())
            .enumerate()
        {
            if matches!(entry, ColumnEntry::Null) && !col.is_nullable() {
                return Err(SchemaError::MissingRequired {
                    column: col.name().to_string(),
                    index: i,
                });
            }
        }
        Ok(EntryArray {
            entries: self.entries,
        })
    }

    /// Finish building the entry array without validation.
    pub fn finish_partial(self) -> EntryArray {
        EntryArray {
            entries: self.entries,
        }
    }
}
