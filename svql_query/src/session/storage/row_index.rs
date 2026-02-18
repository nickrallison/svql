//! Opaque row index type.
//!
//! `RowIndex` is intentionally not constructible or readable outside
//! the `storage` module. All external access goes through `Ref<T>`.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowIndex(u32);

impl RowIndex {
    #[inline]
    pub(super) const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }

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
