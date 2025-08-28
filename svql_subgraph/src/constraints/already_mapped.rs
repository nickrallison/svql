use crate::Constraint;
use crate::isomorphism::NodeMapping;
use prjunnamed_netlist::CellRef;

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
        !self.node_mapping.design_mapping().contains_key(node)
    }
}
