use svql_macros::netlist;

use crate::{State, Wire};

netlist! {
    name: OrGate,
    module_name: "or_gate",
    file: "examples/patterns/basic/or/verilog/or_gate.v",
    inputs: [a, b],
    outputs: [y]
}

impl<S> OrGate<S>
where
    S: State,
{
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone(), self.b.clone()]
    }
}
