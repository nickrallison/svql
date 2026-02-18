//! Typed metadata values for pattern result columns.
//!
//! Replaces the raw `u32` previously used in `ColumnEntry::Metadata`,
//! distinguishing between cell references, variant discriminants, and counters.

use std::fmt;
use svql_common::PhysicalCellId;

/// An opaque index into a `Variant`'s `VARIANT_ARMS` array.
///
/// Cannot be constructed outside the `session` module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariantIndex(u32);

impl VariantIndex {
    /// Construct from a raw index. Only available within the `session` module.
    pub(crate) const fn new(idx: u32) -> Self {
        Self(idx)
    }

    /// Read the raw value.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for VariantIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "variant({})", self.0)
    }
}

/// A typed value stored in a metadata column.
///
/// This replaces the raw `u32` that was previously used in
/// `ColumnEntry::Metadata`, attaching semantic meaning to the integer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MetaValue {
    /// A reference to a physical cell in the haystack design.
    /// Used by `Netlist` patterns to store internal cell mappings.
    CellRef(PhysicalCellId),
    /// A discriminant index selecting which arm of a `Variant` was matched.
    Discriminant(VariantIndex),
    /// An unsigned integer counter (e.g., tree depth in `Recursive` patterns).
    Count(u32),
}

impl MetaValue {
    /// Extract the cell reference if this is `CellRef`.
    #[must_use]
    pub const fn as_cell_ref(&self) -> Option<PhysicalCellId> {
        match self {
            Self::CellRef(id) => Some(*id),
            _ => None,
        }
    }

    /// Extract the variant discriminant index if this is `Discriminant`.
    #[must_use]
    pub const fn as_discriminant(&self) -> Option<VariantIndex> {
        match self {
            Self::Discriminant(idx) => Some(*idx),
            _ => None,
        }
    }

    /// Extract the count if this is `Count`.
    #[must_use]
    pub const fn as_count(&self) -> Option<u32> {
        match self {
            Self::Count(n) => Some(*n),
            _ => None,
        }
    }
}

impl fmt::Display for MetaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CellRef(id) => write!(f, "cell({})", id),
            Self::Discriminant(idx) => write!(f, "{}", idx),
            Self::Count(n) => write!(f, "count({})", n),
        }
    }
}
