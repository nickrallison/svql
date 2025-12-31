use crate::State;
use crate::primitives::dff::{Dff, Dffe};
use svql_macros::variant;

#[variant(ports(clk, data_in, data_out))]
pub enum Cwe1271<S: State> {
    #[variant(map(clk = "clk", data_in = "d", data_out = "q"))]
    WithEnable(Dffe<S>),

    #[variant(map(clk = "clk", data_in = "d", data_out = "q"))]
    Basic(Dff<S>),
}
