use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg_en.v",
    name = "uninit_reg_en"
)]
pub struct UninitRegEn<S: State> {
    pub clk: Wire<S>,
    pub data_in: Wire<S>,
    pub write_en: Wire<S>,
    pub data_out: Wire<S>,
}

#[netlist(
    file = "examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg.v",
    name = "uninit_reg"
)]
pub struct UninitReg<S: State> {
    pub clk: Wire<S>,
    pub data_in: Wire<S>,
    pub data_out: Wire<S>,
}
