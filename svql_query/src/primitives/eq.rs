use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/basic/eq/verilog/eq_gate.v",
    name = "eq_gate"
)]
pub struct EqGate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}
