use svql_common::Wire;

use super::meta_value::MetaValue;
use super::row_index::RowIndex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ColumnEntry {
    Null,
    Wire(Wire),
    WireArray(Vec<Wire>),
    Sub(RowIndex),
    Meta(MetaValue),
}

impl ColumnEntry {
    pub fn as_wire(&self) -> Option<&Wire> {
        match self {
            Self::Wire(w) => Some(w),
            _ => None,
        }
    }

    pub fn as_wire_array(&self) -> Option<&[Wire]> {
        match self {
            Self::WireArray(ws) => Some(ws),
            _ => None,
        }
    }

    pub fn as_meta(&self) -> Option<&MetaValue> {
        match self {
            Self::Meta(m) => Some(m),
            _ => None,
        }
    }

    pub(super) fn as_row_index(&self) -> Option<RowIndex> {
        match self {
            Self::Sub(idx) => Some(*idx),
            _ => None,
        }
    }

    /// Returns the raw row index for type-erased sub-entry access.
    pub(crate) fn as_sub_raw(&self) -> Option<u32> {
        self.as_row_index().map(|idx| idx.raw())
    }
}
