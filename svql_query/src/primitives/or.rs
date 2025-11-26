use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/basic/or/verilog/or_gate.v",
    name = "or_gate"
)]
pub struct OrGate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S: State> OrGate<S> {
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone(), self.b.clone()]
    }
}
