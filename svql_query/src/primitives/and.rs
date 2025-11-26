use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/basic/and/verilog/and_gate.v",
    name = "and_gate"
)]
pub struct AndGate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

#[netlist(
    file = "examples/patterns/basic/and/verilog/and_mux.v",
    name = "and_mux"
)]
pub struct AndMux<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

#[netlist(
    file = "examples/patterns/basic/and/verilog/and_nor.v",
    name = "and_nor"
)]
pub struct AndNor<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S: State> AndGate<S> {
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone(), self.b.clone()]
    }
}
