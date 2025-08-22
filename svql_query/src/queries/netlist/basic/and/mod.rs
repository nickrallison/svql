use crate::netlist;

// pub mod and_gate;
pub mod and_mux;
pub mod and_nor;

netlist! {
    name: And,
    module_name: "and_gate",
    file: "examples/patterns/basic/and/and_gate.v",
    inputs: [a, b],
    outputs: [y]
}
