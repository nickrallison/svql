//! Type-safe wrapper for cell IDs in hardware designs.
//!
//! `CellId` is a wrapper around `u32` that provides type safety and
//! compatibility with Polars DataFrame indexing.

use std::fmt;

/// A type-safe identifier for cells in a hardware design.
///
/// Internally uses `u32` to match Polars DataFrame indexing requirements.
/// This provides compile-time guarantees that cell IDs aren't confused
/// with other numeric values.
///
/// # Layout
///
/// Internally just a `u32`. The `repr(transparent)` ensures zero-cost abstraction.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellId(u32);

impl CellId {
    /// Create a new CellId from a u32.
    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw u32 value.
    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Convert to u64 for compatibility with existing code.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }

    /// Convert to usize for vector/slice indexing.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Create from u64 (panics if > u32::MAX in debug builds).
    #[inline]
    pub fn from_u64(id: u64) -> Self {
        debug_assert!(id <= u32::MAX as u64, "CellId overflow: {} > u32::MAX", id);
        Self(id as u32)
    }

    /// Create from usize (panics if > u32::MAX in debug builds).
    #[inline]
    pub fn from_usize(id: usize) -> Self {
        debug_assert!(
            id <= u32::MAX as usize,
            "CellId overflow: {} > u32::MAX",
            id
        );
        Self(id as u32)
    }
}

impl fmt::Debug for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CellId({})", self.0)
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c{}", self.0)
    }
}

// Conversions from integer types
impl From<u32> for CellId {
    #[inline]
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<CellId> for u32 {
    #[inline]
    fn from(id: CellId) -> Self {
        id.0
    }
}

impl From<CellId> for u64 {
    #[inline]
    fn from(id: CellId) -> Self {
        id.0 as u64
    }
}

impl From<CellId> for usize {
    #[inline]
    fn from(id: CellId) -> Self {
        id.0 as usize
    }
}

// Polars integration: allow collecting CellId into DataFrame columns
impl From<CellId> for i64 {
    #[inline]
    fn from(id: CellId) -> Self {
        id.0 as i64
    }
}

impl Default for CellId {
    /// Default to cell ID 0.
    #[inline]
    fn default() -> Self {
        Self(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_id_creation() {
        let id = CellId::new(42);
        assert_eq!(id.raw(), 42);
        assert_eq!(id.as_u64(), 42);
        assert_eq!(id.as_usize(), 42);
    }

    #[test]
    fn test_cell_id_conversions() {
        let id: CellId = 42u32.into();
        let back: u32 = id.into();
        assert_eq!(back, 42);

        let u64_val: u64 = id.into();
        assert_eq!(u64_val, 42);

        let usize_val: usize = id.into();
        assert_eq!(usize_val, 42);
    }

    #[test]
    fn test_cell_id_from_u64() {
        let id = CellId::from_u64(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_cell_id_from_usize() {
        let id = CellId::from_usize(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn test_cell_id_display() {
        let id = CellId::new(42);
        assert_eq!(format!("{}", id), "c42");
        assert_eq!(format!("{:?}", id), "CellId(42)");
    }

    #[test]
    fn test_cell_id_ordering() {
        let id1 = CellId::new(1);
        let id2 = CellId::new(2);
        assert!(id1 < id2);
        assert!(id2 > id1);
    }

    #[test]
    fn test_cell_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(CellId::new(1));
        set.insert(CellId::new(2));
        set.insert(CellId::new(1)); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    #[should_panic(expected = "CellId overflow")]
    #[cfg(debug_assertions)]
    fn test_cell_id_overflow_from_u64() {
        let _ = CellId::from_u64(u64::MAX);
    }
}
