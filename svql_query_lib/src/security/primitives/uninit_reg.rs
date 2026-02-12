//! Models for registers lacking proper reset or initialization.

use svql_query::prelude::*;

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg_en.v",
    module = "uninit_reg_en"
)]
/// A register with an enable signal lacking initialization logic.
pub struct UninitRegEn {
    /// The clock signal.
    #[port(input)]
    pub clk: Wire,
    /// The data input.
    #[port(input)]
    pub data_in: Wire,
    /// The write enable signal.
    #[port(input)]
    pub write_en: Wire,
    /// The registered output.
    #[port(output)]
    pub data_out: Wire,
}

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg.v",
    module = "uninit_reg"
)]
/// A basic register lacking initialization logic.
pub struct UninitReg {
    /// The clock signal.
    #[port(input)]
    pub clk: Wire,
    /// The data input.
    #[port(input)]
    pub data_in: Wire,
    /// The registered output.
    #[port(output)]
    pub data_out: Wire,
}
