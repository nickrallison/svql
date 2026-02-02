use svql_query::prelude::*;

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg_en.v",
    module = "uninit_reg_en"
)]
pub struct UninitRegEn {
    #[port(input)]
    pub clk: Wire,
    #[port(input)]
    pub data_in: Wire,
    #[port(input)]
    pub write_en: Wire,
    #[port(output)]
    pub data_out: Wire,
}

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg.v",
    module = "uninit_reg"
)]
pub struct UninitReg {
    #[port(input)]
    pub clk: Wire,
    #[port(input)]
    pub data_in: Wire,
    #[port(output)]
    pub data_out: Wire,
}
