use crate::{State, impl_dff_primitive};
use svql_macros::variant;

fn ff_matches(ff: &prjunnamed_netlist::FlipFlop) -> bool {
    let is_valid_cwe_1271 = !ff.has_reset() && !ff.has_clear();
    if is_valid_cwe_1271 {
        println!("CWE-1271 match: {:?}", ff);
        return true;
    }
    false
}

impl_dff_primitive!(
    DffCwe1271,
    [clk, d, q],
    |ff| ff_matches(ff),
    "Matches basic flip-flops with no reset logic."
);

#[variant(ports(clk, data_in, data_out))]
pub enum Cwe1271<S: State> {
    #[variant(map(clk = "clk", data_in = "d", data_out = "q"))]
    Cwe1271Inst(DffCwe1271<S>),
    // #[variant(map(clk = "clk", data_in = "d", data_out = "q"))]
    // Basic(Dff<S>),
}
