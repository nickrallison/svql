use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/basic/buf/verilog/buf_gate.v",
    name = "buf_gate"
)]
pub struct BufGate<S: State> {
    pub a: Wire<S>,
    pub y: Wire<S>,
}

impl<S: State> BufGate<S> {
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone()]
    }
}
