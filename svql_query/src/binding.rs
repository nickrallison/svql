use crate::Match;
use svql_subgraph::Embedding;

// Macro‑friendly helpers that bind a single bit of a named input or output
// into our `Match<'p, 'd>` payload. A codegen macro can call these for each
// declared port (and iterate width as needed if/when multi‑bit ports are
// supported at the query layer).

pub fn bind_input<'ctx>(m: &Embedding<'ctx, 'ctx>, name: &str, bit: usize) -> Match<'ctx> {
    let pat = m.input_fanout_by_name.get(name).cloned().unwrap();
    let pat_first = &pat[0].0;
    let des = m.mapping.get_design_cell(pat_first.clone()).unwrap();
    return Match {
        pat_node_ref: Some(pat_first.clone()),
        design_node_ref: Some(des),
    };
    todo!("how to handle length matching with bind")
    // todo!()
}

pub fn bind_output<'ctx>(m: &Embedding<'ctx, 'ctx>, name: &str, bit: usize) -> Match<'ctx> {
    let pat = m.output_fanin_by_name.get(name).cloned().unwrap();
    let pat_first = &pat[0].0;
    let des = m.mapping.get_design_cell(pat_first.clone()).unwrap();
    return Match {
        pat_node_ref: Some(pat_first.clone()),
        design_node_ref: Some(des),
    };
    todo!("how to handle length matching with bind")
}
