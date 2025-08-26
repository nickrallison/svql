use crate::Match;
use svql_subgraph::SubgraphMatch;

// Macro‑friendly helpers that bind a single bit of a named input or output
// into our `Match<'p, 'd>` payload. A codegen macro can call these for each
// declared port (and iterate width as needed if/when multi‑bit ports are
// supported at the query layer).

pub fn bind_input<'ctx>(m: &SubgraphMatch<'ctx, 'ctx>, name: &str, bit: usize) -> Match<'ctx> {
    let pat = m.input_by_name.get(name).cloned();
    let des = m.design_source_of_input_bit(name, bit).map(|(c, _b)| c);
    Match {
        pat_cell_ref: pat,
        design_cell_ref: des,
    }
}

pub fn bind_output<'ctx>(m: &SubgraphMatch<'ctx, 'ctx>, name: &str, bit: usize) -> Match<'ctx> {
    let pat = m.output_by_name.get(name).cloned();
    let des = m.design_driver_of_output_bit(name, bit).map(|(c, _b)| c);
    Match {
        pat_cell_ref: pat,
        design_cell_ref: des,
    }
}
