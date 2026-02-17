use contracts::*;
use std::fmt;
use std::hash::Hash;

/// Local identifier within a specific GraphIndex array.
/// Exclusively used inside the subgraph solver for performance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GraphNodeIdx {
    /// The underlying raw integer index.
    inner: u32,
}

impl GraphNodeIdx {
    /// Creates a new local graph index from a raw integer.
    #[ensures(ret.inner == id)]
    pub const fn new(id: u32) -> Self {
        Self { inner: id }
    }

    /// Returns the index as a usize for array access.
    #[ensures(ret == self.inner as usize)]
    pub const fn as_usize(self) -> usize {
        self.inner as usize
    }
}

impl fmt::Display for GraphNodeIdx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "n{}", self.inner)
    }
}

impl From<usize> for GraphNodeIdx {
    #[inline]
    fn from(id: usize) -> Self {
        Self { inner: id as u32 }
    }
}

impl From<GraphNodeIdx> for usize {
    #[inline]
    fn from(id: GraphNodeIdx) -> Self {
        id.inner as Self
    }
}

impl From<GraphNodeIdx> for u32 {
    #[inline]
    fn from(id: GraphNodeIdx) -> Self {
        id.inner
    }
}
