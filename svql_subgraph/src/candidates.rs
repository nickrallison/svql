// // File: svql_subgraph/src/candidates.rs

// use prjunnamed_netlist::CellRef;
// use std::collections::HashSet;
// use std::slice::Iter as SliceIter;

// /// Enum representing either constrained or unconstrained candidate nodes.
// pub enum Candidates<'a, 'd> {
//     Constrained(HashSet<CellRef<'d>>),
//     Unconstrained(&'a [CellRef<'d>]),
// }

// /// A concrete iterator enum that avoids boxing
// pub enum CandidatesIter<'a, 'd> {
//     Constrained(std::collections::hash_set::Iter<'a, CellRef<'d>>),
//     Unconstrained(SliceIter<'a, CellRef<'d>>),
// }

// impl<'a, 'd> Iterator for CandidatesIter<'a, 'd> {
//     type Item = CellRef<'d>;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             CandidatesIter::Constrained(iter) => iter.next().copied(),
//             CandidatesIter::Unconstrained(iter) => iter.next().copied(),
//         }
//     }
// }

// impl<'a, 'd> Candidates<'a, 'd> {
//     /// Get a reference-based iterator (for filtering, etc.)
//     pub fn iter(&self) -> CandidatesIter<'_, 'd> {
//         match self {
//             Candidates::Constrained(set) => CandidatesIter::Constrained(set.iter()),
//             Candidates::Unconstrained(slice) => CandidatesIter::Unconstrained(slice.iter()),
//         }
//     }
// }

use prjunnamed_netlist::CellRef;

use crate::{
    Constraint,
    constraints::{ConnectivityConstraint, NotAlreadyMappedConstraint},
};

/// A small, owned, enum-based iterator over design candidates, with constraints applied lazily.
pub(crate) struct FilteredCandidates<'a, 'p, 'd> {
    pub(crate) base: BaseIter<'a, 'd>,
    pub(crate) already_mapped: NotAlreadyMappedConstraint<'p, 'd>,
    pub(crate) connectivity: ConnectivityConstraint<'a, 'p, 'd>,
}

pub(crate) enum BaseIter<'a, 'd> {
    Constrained(std::collections::hash_set::IntoIter<CellRef<'d>>),
    Unconstrained(std::slice::Iter<'a, CellRef<'d>>),
}

impl<'a, 'd> BaseIter<'a, 'd> {
    fn next_cell(&mut self) -> Option<CellRef<'d>> {
        match self {
            BaseIter::Constrained(it) => it.next(),
            BaseIter::Unconstrained(it) => it.next().copied(),
        }
    }
}

impl<'a, 'p, 'd> Iterator for FilteredCandidates<'a, 'p, 'd> {
    type Item = CellRef<'d>;

    fn next(&mut self) -> Option<Self::Item> {
        // Pull from the base iterator until a candidate passes all constraints.
        while let Some(d) = self.base.next_cell() {
            if !self.connectivity.d_candidate_is_valid(&d) {
                continue;
            }
            if !self.already_mapped.d_candidate_is_valid(&d) {
                continue;
            }
            return Some(d);
        }
        None
    }
}
