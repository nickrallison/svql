use crate::Constraint;
use crate::Timer;
use crate::node_mapping::NodeMapping;
use prjunnamed_netlist::CellRef;
use tracing::trace;

pub(crate) struct NotAlreadyMappedConstraint<'p, 'd> {
    node_mapping: NodeMapping<'p, 'd>,
}

impl<'p, 'd> NotAlreadyMappedConstraint<'p, 'd> {
    pub(crate) fn new(node_mapping: NodeMapping<'p, 'd>) -> Self {
        NotAlreadyMappedConstraint { node_mapping }
    }
}

impl<'p, 'd> Constraint<'d> for NotAlreadyMappedConstraint<'p, 'd> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool {
        let _t = Timer::new("NotAlreadyMappedConstraint::d_candidate_is_valid");
        let is_valid = !self.node_mapping.design_mapping().contains_key(node);
        if !is_valid {
            trace!("Design node {:?} already mapped, rejecting", node);
        }
        is_valid
    }
}
