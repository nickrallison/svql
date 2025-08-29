use prjunnamed_netlist::CellRef;

use crate::{
    Constraint,
    constraints::{ConnectivityConstraint, NotAlreadyMappedConstraint},
    progress::Progress,
};

/// A small, owned, enum-based iterator over design candidates, with constraints applied lazily.
pub(crate) struct FilteredCandidates<'a, 'p, 'd, 'g> {
    pub(crate) base: BaseIter<'a, 'd>,
    pub(crate) already_mapped: NotAlreadyMappedConstraint<'p, 'd>,
    pub(crate) connectivity: ConnectivityConstraint<'a, 'p, 'd>,
    pub(crate) progress: Option<&'g Progress>,
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

impl<'a, 'p, 'd, 'g> Iterator for FilteredCandidates<'a, 'p, 'd, 'g> {
    type Item = CellRef<'d>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(d) = self.base.next_cell() {
            // Count every candidate we consider (even if later rejected).
            if let Some(p) = self.progress {
                p.inc_scanned(1);
            }

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
