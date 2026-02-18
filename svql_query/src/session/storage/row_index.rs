//! Opaque row index type.
//!
//! `RowIndex` is intentionally not constructible or readable outside
//! the `storage` module. All external access goes through `Ref<T>`.

use std::fmt;

/// Opaque index type for rows in a table.
/// This is intentionally not constructible or readable outside the `storage` module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowIndex(u32);

impl RowIndex {
    /// Construct a new `RowIndex` from a raw u32 value.
    #[inline]
    pub(super) const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Extract the raw u32 value from this `RowIndex`.
    #[inline]
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }

    /// Construct a `RowIndex` from a raw u32 value (internal crate use).
    #[inline]
    pub(crate) const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Construct a `RowIndex` from a raw row number.
    ///
    /// Available to external implementors of `Recursive::build_recursive`.
    #[inline]
    pub const fn from_u32(raw: u32) -> Self {
        Self(raw)
    }
}

impl fmt::Display for RowIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
