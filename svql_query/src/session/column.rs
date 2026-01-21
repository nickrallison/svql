//! Column schema definitions for DataFrame-based pattern storage.
//!
//! These types define the schema for pattern result tables, including
//! wire references, submodule references, and metadata columns.

use std::any::TypeId;

/// The kind of data stored in a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnKind {
    /// A wire reference (CellId) pointing into the design.
    Wire,
    /// A submodule reference (Ref<T>) pointing into another pattern table.
    /// For self-referential types (like RecOr trees), use `Sub(TypeId::of::<Self>())`.
    Sub(TypeId),
    /// Metadata column (e.g., depth, flags) - not a reference to other data.
    Metadata,
}

impl ColumnKind {
    /// Create a `Sub` column kind for a specific pattern type.
    pub fn sub<T: 'static>() -> Self {
        Self::Sub(TypeId::of::<T>())
    }

    /// Check if this is a wire column.
    pub fn is_wire(&self) -> bool {
        matches!(self, Self::Wire)
    }

    /// Check if this is a submodule reference column.
    pub fn is_sub(&self) -> bool {
        matches!(self, Self::Sub(_))
    }

    /// Check if this is a metadata column.
    pub fn is_metadata(&self) -> bool {
        matches!(self, Self::Metadata)
    }

    /// Get the target TypeId for a Sub column, if applicable.
    pub fn sub_type(&self) -> Option<TypeId> {
        match self {
            Self::Sub(tid) => Some(*tid),
            _ => None,
        }
    }
}

/// Definition of a single column in a pattern's result table.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Column name (must be unique within a pattern).
    pub name: &'static str,
    /// The kind of data stored in this column.
    pub kind: ColumnKind,
    /// Whether this column can contain NULL values.
    /// Used for optional submodules and tree children (left_child, right_child).
    pub nullable: bool,
}

impl ColumnDef {
    /// Create a wire column (non-nullable by default).
    pub const fn wire(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Wire,
            nullable: false,
        }
    }

    /// Create a wire column that can be NULL.
    pub const fn wire_nullable(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Wire,
            nullable: true,
        }
    }

    /// Create a metadata column (non-nullable by default).
    pub const fn metadata(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Metadata,
            nullable: false,
        }
    }

    /// Create a submodule reference column.
    /// Note: Cannot be `const` because `TypeId::of::<T>()` is not const-stable.
    pub fn sub<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Sub(TypeId::of::<T>()),
            nullable: false,
        }
    }

    /// Create a nullable submodule reference column (for optional children).
    pub fn sub_nullable<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Sub(TypeId::of::<T>()),
            nullable: true,
        }
    }

    /// Create a column with explicit kind and nullability.
    pub const fn new(name: &'static str, kind: ColumnKind, nullable: bool) -> Self {
        Self {
            name,
            kind,
            nullable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_kinds() {
        assert!(ColumnKind::Wire.is_wire());
        assert!(!ColumnKind::Wire.is_sub());

        let sub = ColumnKind::sub::<String>();
        assert!(sub.is_sub());
        assert_eq!(sub.sub_type(), Some(TypeId::of::<String>()));

        assert!(ColumnKind::Metadata.is_metadata());
    }

    #[test]
    fn test_column_def_constructors() {
        let wire = ColumnDef::wire("clk");
        assert_eq!(wire.name, "clk");
        assert!(wire.kind.is_wire());
        assert!(!wire.nullable);

        let nullable_wire = ColumnDef::wire_nullable("optional_port");
        assert!(nullable_wire.nullable);

        let meta = ColumnDef::metadata("depth");
        assert!(meta.kind.is_metadata());
    }
}
