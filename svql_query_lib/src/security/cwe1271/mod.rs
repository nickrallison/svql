use svql_query::prelude::*;

svql_query::define_dff_primitive!(
    DffCwe1271,
    [(clk, input), (d, input), (q, output)],
    |cell| {
        if let prjunnamed_netlist::Cell::Dff(ff) = cell {
            !ff.has_reset() && !ff.has_clear()
        } else {
            false
        }
    }
);

/// Pattern identifying uninitialized values on reset in security-sensitive registers.
#[derive(Debug, Clone, Variant)]
#[variant_ports(input(clk), input(data_in), output(data_out))]
pub enum Cwe1271 {
    /// Instance of the uninitialized DFF matcher.
    #[map(clk = ["clk"], data_in = ["d"], data_out = ["q"])]
    Cwe1271Inst(DffCwe1271),
}
