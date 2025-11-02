use svql_macros::netlist;

use crate::{State, Wire};

netlist! {
    name: Mux2Gate,
    module_name: "mux2_gate",
    file: "examples/patterns/basic/mux2/verilog/mux2_gate.v",
    inputs: [a, b, sel],
    outputs: [y]
}

impl<S> Mux2Gate<S>
where
    S: State,
{
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone(), self.b.clone(), self.sel.clone()]
    }
}
