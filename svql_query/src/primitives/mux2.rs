use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/basic/mux2/verilog/mux2_gate.v",
    name = "mux2_gate"
)]
pub struct Mux2Gate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub sel: Wire<S>,
    pub y: Wire<S>,
}

impl<S: State> Mux2Gate<S> {
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone(), self.b.clone(), self.sel.clone()]
    }
}
