use crate::State;
use crate::security::primitives::uninit_reg::{UninitReg, UninitRegEn};
use svql_macros::variant;

#[variant(ports(clk, data_in, data_out))]
pub enum Cwe1271<S: State> {
    #[variant(map(clk = "clk", data_in = "data_in", data_out = "data_out"))]
    WithEnable(UninitRegEn<S>),

    #[variant(map(clk = "clk", data_in = "data_in", data_out = "data_out"))]
    Basic(UninitReg<S>),
}
