mod connectivity;

use crate::isomorphism::NodeMapping;
use crate::node::NodeType;
use prjunnamed_netlist::CellRef;
use std::collections::HashSet;

pub(super) use connectivity::ConnectivityConstraint;

pub(super) trait Constraint<'d> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool;
}

pub(super) struct NodeConstraints<'d> {
    // Some -> The intersection of these nodes and another set of nodes will be a valid constraint
    // None -> No restriction, any node is valid
    nodes: Option<HashSet<CellRef<'d>>>,
}

impl<'d> NodeConstraints<'d> {
    pub(super) fn new(nodes: Option<HashSet<CellRef<'d>>>) -> Self {
        NodeConstraints { nodes }
    }
    pub(super) fn intersect(self, other: Self) -> Self {
        match (self.nodes, other.nodes) {
            (Some(a), Some(b)) => NodeConstraints::new(Some(a.intersection(&b).cloned().collect())),
            (Some(a), None) => NodeConstraints::new(Some(a)),
            (None, Some(b)) => NodeConstraints::new(Some(b)),
            (None, None) => NodeConstraints::new(None),
        }
    }
    pub(super) fn intersect_many(sets: impl IntoIterator<Item = Self>) -> Self {
        sets.into_iter()
            .fold(NodeConstraints::new(None), |acc, set| acc.intersect(set))
    }
    pub(super) fn is_none(&self) -> bool {
        self.nodes.is_none()
    }
    pub(super) fn get_candidates(&self) -> Option<&HashSet<CellRef<'d>>> {
        self.nodes.as_ref()
    }
    pub(super) fn get_candidates_owned(self) -> Option<HashSet<CellRef<'d>>> {
        self.nodes
    }
}

impl<'d> Constraint<'d> for NodeConstraints<'d> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool {
        match &self.nodes {
            Some(set) => set.contains(node),
            None => true,
        }
    }
}

pub(super) struct TypeConstraint {
    pattern_node_type: NodeType,
}

impl TypeConstraint {
    pub(super) fn new(pattern_node_type: NodeType) -> Self {
        TypeConstraint { pattern_node_type }
    }
}

impl Constraint<'_> for TypeConstraint {
    fn d_candidate_is_valid(&self, node: &CellRef<'_>) -> bool {
        match self.pattern_node_type {
            NodeType::Input => true,
            _ => NodeType::from(node.get().as_ref()) == self.pattern_node_type,
        }
    }
}

pub(super) struct NotAlreadyMappedConstraint<'a, 'p, 'd> {
    node_mapping: &'a NodeMapping<'p, 'd>,
}

impl<'a, 'p, 'd> NotAlreadyMappedConstraint<'a, 'p, 'd> {
    pub(super) fn new(node_mapping: &'a NodeMapping<'p, 'd>) -> Self {
        NotAlreadyMappedConstraint { node_mapping }
    }
}

impl<'a, 'p, 'd> Constraint<'d> for NotAlreadyMappedConstraint<'a, 'p, 'd> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool {
        !self.node_mapping.design_mapping().contains_key(node)
    }
}
