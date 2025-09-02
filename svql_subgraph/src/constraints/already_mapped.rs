use crate::{CellMapping, Timer, cell::CellWrapper, constraints::Constraint};
use prjunnamed_netlist::CellRef;
use tracing::trace;

pub(crate) struct NotAlreadyMappedConstraint<'p, 'd> {
    cell_mapping: CellMapping<'p, 'd>,
}

impl<'p, 'd> NotAlreadyMappedConstraint<'p, 'd> {
    pub(crate) fn new(cell_mapping: CellMapping<'p, 'd>) -> Self {
        NotAlreadyMappedConstraint { cell_mapping }
    }
}

impl<'p, 'd> Constraint<'d> for NotAlreadyMappedConstraint<'p, 'd> {
    fn d_candidate_is_valid(&self, cell: &CellWrapper<'d>) -> bool {
        let _t = Timer::new("NotAlreadyMappedConstraint::d_candidate_is_valid");
        let is_valid = !self
            .cell_mapping
            .design_mapping()
            .contains_key(&(cell.clone().into()));
        if !is_valid {
            trace!("Design cell {:?} already mapped, rejecting", cell);
        }
        is_valid
    }
}
