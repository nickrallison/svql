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
#[repr(transparent)]
pub struct CellId(u64);

impl CellId {
    /// Design ID for the root/flattened design (single-design mode).
    pub const ROOT_DESIGN: u16 = 0;

    /// Create a `CellId` for the root/flattened design.
    ///
    /// This is the common case for single-design queries.
    #[inline]
    pub const fn new(cell_idx: u32) -> Self {
        Self::with_design(Self::ROOT_DESIGN, cell_idx)
    }

    /// Create a `CellId` with an explicit design ID.
    ///
    /// Use this for multi-design hierarchies where cells come from
    /// different loaded designs.
    #[inline]
    pub const fn with_design(design_id: u16, cell_idx: u32) -> Self {
        Self(((design_id as u64) << 48) | (cell_idx as u64))
    }

    /// Get the design identifier.
    #[inline]
    pub const fn design_id(self) -> u16 {
        (self.0 >> 48) as u16
    }

    /// Get the cell index within the design.
    #[inline]
    pub const fn cell_idx(self) -> u32 {
        self.0 as u32
    }

    /// Check if this cell is from the root/flattened design.
    #[inline]
    pub const fn is_root_design(self) -> bool {
        self.design_id() == Self::ROOT_DESIGN
    }

    /// Get the raw u64 value for DataFrame storage.
    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Reconstruct from a raw DataFrame value.
    #[inline]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

// Convenience: u32 → CellId (assumes root design)
impl From<u32> for CellId {
    #[inline]
    fn from(idx: u32) -> Self {
        Self::new(idx)
    }
}

// Convenience: usize → CellId (assumes root design, saturates at u32::MAX)
impl From<usize> for CellId {
    #[inline]
    fn from(idx: usize) -> Self {
        Self::new(idx.min(u32::MAX as usize) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_design() {
        let id = CellId::new(42);
        assert_eq!(id.design_id(), 0);
        assert_eq!(id.cell_idx(), 42);
        assert!(id.is_root_design());
    }

    #[test]
    fn test_multi_design() {
        let id = CellId::with_design(5, 1000);
        assert_eq!(id.design_id(), 5);
        assert_eq!(id.cell_idx(), 1000);
        assert!(!id.is_root_design());
    }

    #[test]
    fn test_raw_roundtrip() {
        let id = CellId::with_design(0xABCD, 0x12345678);
        let raw = id.raw();
        let restored = CellId::from_raw(raw);
        assert_eq!(id, restored);
        assert_eq!(restored.design_id(), 0xABCD);
        assert_eq!(restored.cell_idx(), 0x12345678);
    }

    #[test]
    fn test_from_u32() {
        let id: CellId = 100u32.into();
        assert_eq!(id.cell_idx(), 100);
        assert!(id.is_root_design());
    }
}
