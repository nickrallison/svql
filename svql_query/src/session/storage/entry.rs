use svql_common::Wire;

use super::meta_value::MetaValue;
use super::row_index::RowIndex;

/// An entry in a column, representing various types of stored data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColumnEntry {
    /// No value present.
    Null,
    /// A single wire reference.
    Wire(Wire),
    /// An array of wire references.
    WireArray(Vec<Wire>),
    /// A reference to a submodule row.
    Sub(RowIndex),
    /// A metadata value.
    Meta(MetaValue),
}

impl ColumnEntry {
    /// Extract the wire reference if this entry contains a wire.
    pub const fn as_wire(&self) -> Option<&Wire> {
        match self {
            Self::Wire(w) => Some(w),
            _ => None,
        }
    }

    /// Extract the wire array slice if this entry contains a wire array.
    pub fn as_wire_array(&self) -> Option<&[Wire]> {
        match self {
            Self::WireArray(ws) => Some(ws),
            _ => None,
        }
    }

    /// Extract the metadata value if this entry contains metadata.
    pub const fn as_meta(&self) -> Option<&MetaValue> {
        match self {
            Self::Meta(m) => Some(m),
            _ => None,
        }
    }

    /// Extract the row index if this entry contains a sub-row reference.
    #[allow(dead_code)]
    pub(super) const fn as_row_index(&self) -> Option<RowIndex> {
        match self {
            Self::Sub(idx) => Some(*idx),
            _ => None,
        }
    }

    /// Returns the raw row index for type-erased sub-entry access.
    #[allow(dead_code)]
    pub(crate) const fn as_sub_raw(&self) -> Option<u32> {
        match self {
            Self::Sub(idx) => Some(idx.raw()),
            _ => None,
        }
    }
}
