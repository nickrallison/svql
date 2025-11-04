use svql_macros::netlist;

netlist! {
    name: Sdffe,
    module_name: "sdffe",
    file: "examples/patterns/basic/ff/verilog/sdffe.v",
    inputs: [clk, d, reset, en],
    outputs: [q]
}

netlist! {
    name: Adffe,
    module_name: "adffe",
    file: "examples/patterns/basic/ff/rtlil/adffe.il",
    inputs: [clk, d, reset_n, en],
    outputs: [q]
}
netlist! {
    name: Sdff,
    module_name: "sdff",
    file: "examples/patterns/basic/ff/rtlil/sdff.il",
    inputs: [clk, d, reset],
    outputs: [q]
}

netlist! {
    name: Adff,
    module_name: "adff",
    file: "examples/patterns/basic/ff/rtlil/adff.il",
    inputs: [clk, d, reset_n],
    outputs: [q]
}
