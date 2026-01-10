use crate::{State, impl_dff_primitive};
use svql_macros::variant;

impl_dff_primitive!(
    DffCwe1271,
    [clk, d, q],
    |ff| !ff.has_reset() && !ff.has_clear(),
    "Matches basic flip-flops with no reset logic."
);

#[variant(ports(clk, data_in, data_out))]
pub enum Cwe1271<S: State> {
    #[variant(map(clk = "clk", data_in = "d", data_out = "q"))]
    Cwe1271Inst(DffCwe1271<S>),
    // #[variant(map(clk = "clk", data_in = "d", data_out = "q"))]
    // Basic(Dff<S>),
}
