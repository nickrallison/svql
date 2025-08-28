use crate::Constraint;
use crate::isomorphism::NodeMapping;
use prjunnamed_netlist::CellRef;

pub(crate) struct NotAlreadyMappedConstraint<'a, 'p, 'd> {
    node_mapping: &'a NodeMapping<'p, 'd>,
}

impl<'a, 'p, 'd> NotAlreadyMappedConstraint<'a, 'p, 'd> {
    pub(crate) fn new(node_mapping: &'a NodeMapping<'p, 'd>) -> Self {
        NotAlreadyMappedConstraint { node_mapping }
    }
}

impl<'a, 'p, 'd> Constraint<'d> for NotAlreadyMappedConstraint<'a, 'p, 'd> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool {
        !self.node_mapping.design_mapping().contains_key(node)
    }
}
