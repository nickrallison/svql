use svql_macros::netlist;

use crate::{State, Wire};

netlist! {
    name: XnorGate,
    module_name: "xnor_gate",
    file: "examples/patterns/basic/xnor/verilog/xnor_gate.v",
    inputs: [a, b],
    outputs: [y]
}

impl<S> XnorGate<S>
where
    S: State,
{
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone(), self.b.clone()]
    }
}
