use prjunnamed_netlist::CellRef;

use crate::{Constraint, node::NodeType};

pub(crate) struct TypeConstraint {
    pattern_node_type: NodeType,
}

impl TypeConstraint {
    pub(crate) fn new(pattern_node_type: NodeType) -> Self {
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
