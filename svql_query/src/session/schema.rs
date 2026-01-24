//! Column schema definitions for DataFrame-based pattern storage.
//!
//! These types define the schema for pattern result tables, including
//! wire references, submodule references, and metadata columns.

use std::any::TypeId;

/// The kind of data stored in a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnKind {
    /// A wire reference (CellId) pointing into the design.
    Cell,
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
        matches!(self, Self::Cell)
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

/// Defines the direction of a port column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDirection {
    /// Not a port (internal wire, submodule reference, or metadata).
    None,
    /// Input port (receives signal).
    Input,
    /// Output port (drives signal).
    Output,
    /// Bidirectional port.
    Inout,
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
    /// The directionality of this column (if it is a port).
    pub direction: PortDirection,
}

impl ColumnDef {
    /// Create a wire column that acts as an Input port.
    pub const fn input(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Cell,
            nullable: false,
            direction: PortDirection::Input,
        }
    }

    /// Create a wire column that acts as an Output port.
    pub const fn output(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Cell,
            nullable: false,
            direction: PortDirection::Output,
        }
    }

    /// Create a wire column (non-nullable by default).
    pub const fn wire(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Cell,
            nullable: false,
            direction: PortDirection::None,
        }
    }

    /// Create a wire column that can be NULL.
    pub const fn wire_nullable(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Cell,
            nullable: true,
            direction: PortDirection::None,
        }
    }

    /// Create a metadata column (non-nullable by default).
    pub const fn metadata(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Metadata,
            nullable: false,
            direction: PortDirection::None,
        }
    }

    /// Create a submodule reference column.
    /// Note: Cannot be `const` because `TypeId::of::<T>()` is not const-stable.
    pub const fn sub<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Sub(TypeId::of::<T>()),
            nullable: false,
            direction: PortDirection::None,
        }
    }

    /// Create a nullable submodule reference column (for optional children).
    pub fn sub_nullable<T: 'static>(name: &'static str) -> Self {
        Self {
            name,
            kind: ColumnKind::Sub(TypeId::of::<T>()),
            nullable: true,
            direction: PortDirection::None,
        }
    }

    /// Create a column with explicit kind and nullability.
    pub const fn new(name: &'static str, kind: ColumnKind, nullable: bool) -> Self {
        Self {
            name,
            kind,
            nullable,
            direction: PortDirection::None,
        }
    }

    pub fn into_polars_column(&self) -> polars::frame::column::Column {
        use polars::prelude::*;

        let d_type = match &self.kind {
            ColumnKind::Cell => DataType::UInt64,
            ColumnKind::Sub(tid) => DataType::UInt32,
            ColumnKind::Metadata => DataType::UInt64,
        };
        Column::new_empty(PlSmallStr::from_static(self.name), &d_type)
    }
}

#[derive(Debug, Clone)]
pub enum ColumnEntry {
    Cell { id: Option<u64> },
    Sub { id: Option<u64> },
    Metadata { id: Option<u64> },
}

impl ColumnEntry {
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            ColumnEntry::Cell { id } => *id,
            ColumnEntry::Sub { id } => *id,
            ColumnEntry::Metadata { id } => *id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntryArray {
    pub entries: Vec<ColumnEntry>,
}

impl EntryArray {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: vec![ColumnEntry::Metadata { id: None }; capacity],
        }
    }
    pub(crate) fn new(entries: Vec<ColumnEntry>) -> Self {
        Self { entries }
    }
}
