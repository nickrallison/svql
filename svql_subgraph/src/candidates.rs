use std::collections::HashSet;
use std::slice::Iter as SliceIter;

use prjunnamed_netlist::CellRef;

/// Enum representing either constrained or unconstrained candidate nodes.
pub enum Candidates<'a, 'd> {
    Constrained(HashSet<CellRef<'d>>),
    Unconstrained(&'a [CellRef<'d>]),
}

/// An iterator over `CellRef<'d>` from either variant of `Candidates`.
pub enum CandidatesIter<'a, 'd> {
    Constrained(std::collections::hash_set::IntoIter<CellRef<'d>>),
    Unconstrained(SliceIter<'a, CellRef<'d>>),
}

impl<'a, 'd> Iterator for CandidatesIter<'a, 'd> {
    type Item = CellRef<'d>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CandidatesIter::Constrained(iter) => iter.next(),
            CandidatesIter::Unconstrained(iter) => iter.next().copied(),
        }
    }
}

impl<'a, 'd> Candidates<'a, 'd> {
    /// Convert into an iterator over `CellRef<'d>`.
    pub fn into_iter(self) -> CandidatesIter<'a, 'd> {
        match self {
            Candidates::Constrained(set) => CandidatesIter::Constrained(set.into_iter()),
            Candidates::Unconstrained(slice) => CandidatesIter::Unconstrained(slice.iter()),
        }
    }

    /// Get a reference-based iterator (for filtering, etc.)
    pub fn iter(&self) -> CandidatesIter<'_, 'd> {
        match self {
            Candidates::Constrained(set) => CandidatesIter::Constrained(set.clone().into_iter()),
            Candidates::Unconstrained(slice) => CandidatesIter::Unconstrained(slice.iter()),
        }
    }
}
