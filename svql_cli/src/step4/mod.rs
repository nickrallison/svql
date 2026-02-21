use crate::step1::AdcGate;
use crate::step3::FullAdderComposite;
use svql_query::prelude::*;

#[derive(Debug, Clone, Variant)]
#[variant_ports(input(a), input(b), input(cin), output(sum), output(cout))]
pub enum AnyFullAdder {
    #[map(a = ["a"], b = ["b"], cin = ["cin"], sum = ["sum"], cout = ["cout"])]
    Structural(FullAdderComposite),

    #[map(a = ["a"], b = ["b"], cin = ["cin"], sum = ["y"], cout = ["null"])]
    Primitive(AdcGate),
}
