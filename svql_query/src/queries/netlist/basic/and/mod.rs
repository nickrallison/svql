use crate::netlist;

netlist! {
    name: AndGate,
    module_name: "and_gate",
    file: "examples/patterns/basic/and/verilog/and_gate.v",
    inputs: [a, b],
    outputs: [y]
}

netlist! {
    name: AndMux,
    module_name: "and_mux",
    file: "examples/patterns/basic/and/verilog/and_mux.v",
    inputs: [a, b],
    outputs: [y]
}

netlist! {
    name: AndNor,
    module_name: "and_nor",
    file: "examples/patterns/basic/and/verilog/and_nor.v",
    inputs: [a, b],
    outputs: [y]
}
