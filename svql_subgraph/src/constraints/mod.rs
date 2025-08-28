mod already_mapped;
mod connectivity;
mod design_sink;
mod design_source;
mod node;
mod ty;

use prjunnamed_netlist::CellRef;

pub(crate) use already_mapped::NotAlreadyMappedConstraint;
pub(crate) use connectivity::ConnectivityConstraint;
pub(crate) use design_sink::DesignSinkConstraint;
pub(crate) use design_source::DesignSourceConstraint;
pub(crate) use ty::TypeConstraint;

pub(super) trait Constraint<'d> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool;
}
