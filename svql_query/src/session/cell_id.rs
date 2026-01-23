//! Cell identifier with support for multi-design hierarchies.
//!
//! `CellId` is a 64-bit identifier that encodes both design membership and
//! cell index, enabling future multi-design support without API changes.

/// Identifier for a cell within a (possibly nested) design hierarchy.
///
/// Layout: `[design_id: u16][reserved: u16][cell_idx: u32]`
/// - `design_id`: Identifies which design (0 = root/flattened, others for nested)
/// - `reserved`: For future use (e.g., hierarchy level, flags)
/// - `cell_idx`: Index within that design's cell array
///
/// Current single-design mode: `design_id` is always 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellId {
    cell_idx: u64,
}

impl CellId {
    /// Create a `CellId` for the root/flattened design.
    ///
    /// This is the common case for single-design queries.
    #[inline]
    pub const fn new(cell_idx: u64) -> Self {
        Self { cell_idx }
    }

    /// Get the cell index within the design.
    #[inline]
    pub const fn cell_idx(self) -> u64 {
        self.cell_idx
    }
}

// Convenience: u32 → CellId (assumes root design)
impl From<u64> for CellId {
    #[inline]
    fn from(idx: u64) -> Self {
        Self::new(idx)
    }
}

// Convenience: usize → CellId (assumes root design, saturates at u32::MAX)
impl From<usize> for CellId {
    #[inline]
    fn from(idx: usize) -> Self {
        Self::new(idx as u64)
    }
}
