//! Patterns for detecting Clock Domain Crossing (CDC) violations.
use svql_query::prelude::*;
use svql_query_lib::DffAny;
use svql_query_lib::LogicCone;

/// Helper to check if two clock wires are different.
fn check_clocks(src_clk: Option<Wire>, dst_clk: Option<Wire>) -> bool {
    match (src_clk, dst_clk) {
        (Some(s), Some(d)) => s.cell_id() != d.cell_id(),
        _ => false,
    }
}

/// CDC violation via a logic cone between flip-flops.
#[derive(Debug, Clone, Composite)]
#[connection(from = ["source", "q"], to = ["logic_cone", "leaf_inputs"], kind = "any")]
#[connection(from = ["logic_cone", "y"], to = ["dest", "d"])]
#[filter(|row: &Row<Self>, ctx: &ExecutionContext| check_clocks(
    row.resolve(Selector::static_path(&["source", "clk"]), ctx),
    row.resolve(Selector::static_path(&["dest", "clk"]), ctx)
))]
pub struct CdcViolationLogicCone {
    /// The source flip-flop in domain A.
    #[submodule]
    pub source: DffAny,
    /// The combinational logic cone between the flip-flops.
    #[submodule]
    pub logic_cone: LogicCone,
    /// The destination flip-flop in domain B.
    #[submodule]
    pub dest: DffAny,
}

/// CDC violation via a direct connection between flip-flops.
#[derive(Debug, Clone, Composite)]
#[connection(from = ["source", "q"], to = ["dest", "d"])]
#[filter(|row: &Row<Self>, ctx: &ExecutionContext| check_clocks(
    row.resolve(Selector::static_path(&["source", "clk"]), ctx),
    row.resolve(Selector::static_path(&["dest", "clk"]), ctx)
))]
pub struct CdcViolationDirect {
    /// The source flip-flop in domain A.
    #[submodule]
    pub source: DffAny,
    /// The destination flip-flop in domain B.
    #[submodule]
    pub dest: DffAny,
}

/// Detects a Clock Domain Crossing violation where a flip-flop drives
/// another flip-flop on a different clock, either directly or through logic.
#[derive(Debug, Clone, Variant)]
#[variant_ports(input(source_clk), input(dest_clk), output(source_q), input(dest_d))]
pub enum CdcViolation {
    /// Violation via combinational logic.
    #[map(
        source_clk = ["source", "clk"],
        dest_clk = ["dest", "clk"],
        source_q = ["source", "q"],
        dest_d = ["dest", "d"]
    )]
    LogicCone(CdcViolationLogicCone),

    /// Violation via direct connection.
    #[map(
        source_clk = ["source", "clk"],
        dest_clk = ["dest", "clk"],
        source_q = ["source", "q"],
        dest_d = ["dest", "d"]
    )]
    Direct(CdcViolationDirect),
}
