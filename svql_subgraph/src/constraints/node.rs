use std::collections::HashSet;

use prjunnamed_netlist::CellRef;

use crate::Constraint;

pub(crate) struct NodeConstraint<'d> {
    // Some -> The intersection of these nodes and another set of nodes will be a valid constraint
    // None -> No restriction, any node is valid
    nodes: Option<HashSet<CellRef<'d>>>,
}

impl<'d> NodeConstraint<'d> {
    pub(crate) fn new(nodes: Option<HashSet<CellRef<'d>>>) -> Self {
        NodeConstraint { nodes }
    }
    pub(crate) fn intersect(self, other: Self) -> Self {
        match (self.nodes, other.nodes) {
            (Some(a), Some(b)) => NodeConstraint::new(Some(a.intersection(&b).cloned().collect())),
            (Some(a), None) => NodeConstraint::new(Some(a)),
            (None, Some(b)) => NodeConstraint::new(Some(b)),
            (None, None) => NodeConstraint::new(None),
        }
    }
    pub(crate) fn intersect_many(sets: impl IntoIterator<Item = Self>) -> Self {
        sets.into_iter()
            .fold(NodeConstraint::new(None), |acc, set| acc.intersect(set))
    }
    pub(crate) fn is_none(&self) -> bool {
        self.nodes.is_none()
    }
    pub(crate) fn get_candidates(&self) -> Option<&HashSet<CellRef<'d>>> {
        self.nodes.as_ref()
    }
    pub(crate) fn get_candidates_owned(self) -> Option<HashSet<CellRef<'d>>> {
        self.nodes
    }
}

impl<'d> Constraint<'d> for NodeConstraint<'d> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool {
        match &self.nodes {
            Some(set) => set.contains(node),
            None => true,
        }
    }
}
