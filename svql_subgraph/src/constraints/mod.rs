mod already_mapped;
mod connectivity;
// mod design_sink;
// mod design_source;
// mod node;
// mod ty;


// pub(crate) use connectivity::ConnectivityConstraint;

use crate::cell::CellWrapper;
// pub(crate) use design_sink::DesignSinkConstraint;
// pub(crate) use design_source::DesignSourceConstraint;
// pub(crate) use node::NodeConstraint;

pub(super) trait Constraint<'d> {
    fn d_candidate_is_valid(&self, node: &CellWrapper<'d>) -> bool;
}
