use svql_query::prelude::*;
use svql_query_lib::primitives::{DffAny, LogicCone};

fn check_clocks(src_clk: Option<Wire>, dst_clk: Option<Wire>) -> bool {
    match (src_clk, dst_clk) {
        (Some(s), Some(d)) => s.cell_id() != d.cell_id(),
        _ => false,
    }
}

#[derive(Debug, Clone, Composite)]
#[connection(from = ["source", "q"], to = ["logic_cone", "leaf_inputs"], kind = "any")]
#[connection(from = ["logic_cone", "y"], to = ["dest", "d"])]
#[filter(|row: &Row<Self>, ctx: &ExecutionContext| check_clocks(
    row.resolve(Selector::static_path(&["source", "clk"]), ctx),
    row.resolve(Selector::static_path(&["dest", "clk"]), ctx)
))]
pub struct CdcViolation {
    #[submodule]
    pub source: DffAny,
    #[submodule]
    pub logic_cone: LogicCone,
    #[submodule]
    pub dest: DffAny,
}
