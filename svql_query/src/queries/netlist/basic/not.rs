use svql_macros::netlist;

netlist! {
    name: NotGate,
    module_name: "not_gate",
    file: "examples/patterns/basic/not/verilog/not_gate.v",
    inputs: [a],
    outputs: [y]
}
