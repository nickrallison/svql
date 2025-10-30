use svql_macros::netlist;

netlist! {
    name: OrGate,
    module_name: "or_gate",
    file: "examples/patterns/basic/or/verilog/or_gate.v",
    inputs: [a, b],
    outputs: [y]
}
