use svql_macros::netlist;

// ============================================================================
// Pattern Definitions using netlist! macro
// ============================================================================

netlist! {
    name: UninitRegEn,
    module_name: "uninit_reg_en",
    file: "examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg_en.v",
    inputs: [clk, data_in, write_en],
    outputs: [data_out]
}

netlist! {
    name: UninitReg,
    module_name: "uninit_reg",
    file: "examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg.v",
    inputs: [clk, data_in],
    outputs: [data_out]
}
