use contracts::*;
use prjunnamed_netlist::{CellRef, Net, Trit};
use std::fmt;
use std::hash::Hash;

use crate::CellWrapper;

/// Persistent ID from the netlist source (e.g., debug_index from prjunnamed).
/// This is used for storage in Tables and cross-referencing between queries.
///
/// This is recived from the debug_index of the CellRef in prjunnamed, and as such,
/// it is not supposed to be created manually by users, but rather
/// obtained through the API when processing cells and wires.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PhysicalCellId {
    /// The underlying raw integer ID.
    inner: u32,
}

impl PhysicalCellId {
    /// Creates a new persistent cell ID from a raw integer.
    #[ensures(ret.inner == id)]
    pub(crate) const fn new(id: u32) -> Self {
        Self { inner: id }
    }

    /// Access for table packing logic.
    #[ensures(ret == self.inner)]
    pub const fn storage_key(&self) -> u32 {
        self.inner
    }
}

impl From<CellWrapper<'_>> for PhysicalCellId {
    fn from(wrapper: CellWrapper<'_>) -> Self {
        Self::from(wrapper.inner())
    }
}

impl From<CellRef<'_>> for PhysicalCellId {
    fn from(cell_ref: CellRef<'_>) -> Self {
        Self {
            inner: cell_ref.debug_index() as u32,
        }
    }
}

impl TryInto<PhysicalCellId> for Net {
    type Error = Trit;

    fn try_into(self) -> Result<PhysicalCellId, Self::Error> {
        let cell_id = self.as_cell_index().ok().map(|idx| idx as u32);
        cell_id
            .map(PhysicalCellId::new)
            .ok_or(self)
            .map_err(|net| net.as_const().unwrap())
    }
}

impl fmt::Display for PhysicalCellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "p{}", self.inner)
    }
}
