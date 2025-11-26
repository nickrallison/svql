use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/basic/not/verilog/not_gate.v",
    name = "not_gate"
)]
pub struct NotGate<S: State> {
    pub a: Wire<S>,
    pub y: Wire<S>,
}

impl<S: State> NotGate<S> {
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone()]
    }
}
