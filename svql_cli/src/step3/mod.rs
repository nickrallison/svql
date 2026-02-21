use crate::step1::AdcWithCarry;
use crate::step2::HalfAdder;
use svql_query::prelude::*;

#[derive(Debug, Clone, Variant)]
#[variant_ports(input(a), input(b), output(sum), output(carry))]
pub enum AnyHalfAdder {
    #[map(a = ["a"], b = ["b"], sum = ["sum"], carry = ["carry"])]
    Structural(HalfAdder),

    #[map(a = ["a"], b = ["b"], sum = ["sum"], carry = ["carry"])]
    Primitive(AdcWithCarry),
}
