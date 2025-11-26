use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/basic/xnor/verilog/xnor_gate.v",
    name = "xnor_gate"
)]
pub struct XnorGate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S: State> XnorGate<S> {
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone(), self.b.clone()]
    }
}
