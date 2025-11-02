use svql_macros::netlist;

use crate::{State, Wire};

netlist! {
    name: BufGate,
    module_name: "buf_gate",
    file: "examples/patterns/basic/buf/verilog/buf_gate.v",
    inputs: [a],
    outputs: [y]
}

impl<S> BufGate<S>
where
    S: State,
{
    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        vec![self.a.clone()]
    }
}
