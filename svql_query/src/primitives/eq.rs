use svql_macros::netlist;

netlist! {
    name: EqGate,
    module_name: "eq_gate",
    file: "examples/patterns/basic/eq/verilog/eq_gate.v",
    inputs: [a, b],
    outputs: [y]
}