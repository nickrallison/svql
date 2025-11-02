use svql_macros::netlist;

netlist! {
    name: XorGate,
    module_name: "xor_gate",
    file: "examples/patterns/basic/xor/verilog/xor_gate.v",
    inputs: [a, b],
    outputs: [y]
}
