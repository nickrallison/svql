use svql_macros::netlist;

netlist! {
    name: Sdffe,
    module_name: "sdffe",
    file: "examples/patterns/basic/ff/verilog/sdffe.v",
    inputs: [clk, d, reset],
    outputs: [q]
}
