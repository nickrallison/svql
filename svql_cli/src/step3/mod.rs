use crate::step2::FullAdderHierarchical;
use svql_query::prelude::*;

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "svql_cli/src/step3/full_adder_flat.v",
    module = "full_adder_flat"
)]
pub struct FullAdderFlat {
    #[port(input)]
    pub a: Wire,
    #[port(input)]
    pub b: Wire,
    #[port(input)]
    pub cin: Wire,
    #[port(output)]
    pub sum: Wire,
    #[port(output)]
    pub cout: Wire,
}

#[derive(Debug, Clone, Variant)]
#[variant_ports(input(a), input(b), input(cin), output(sum), output(cout))]
#[allow(clippy::large_enum_variant)]
pub enum AnyFullAdder {
    #[map(a = ["a"], b = ["b"], cin = ["cin"], sum = ["sum"], cout = ["cout"])]
    Hierarchical(FullAdderHierarchical),

    #[map(a = ["a"], b = ["b"], cin = ["cin"], sum = ["sum"], cout = ["cout"])]
    Flat(FullAdderFlat),
}
