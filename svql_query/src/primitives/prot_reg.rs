use svql_macros::netlist;

// Primitive: Protected DFF entry (single array element with lock mux).
netlist! {
    name: ProtDffEntry,
    module_name: "and_gate",
    file: "examples/patterns/basic/and/verilog/and_gate.v",
    inputs: [a, b],
    outputs: [y]
}
