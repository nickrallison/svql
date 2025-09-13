use std::collections::HashMap;

use crate::Match;
use svql_subgraph::{Embedding, cell::CellWrapper};

// Macro‑friendly helpers that bind a single bit of a named input or output
// into our `Match<'p, 'd>` payload. A codegen macro can call these for each
// declared port (and iterate width as needed if/when multi‑bit ports are
// supported at the query layer).

pub fn bind_input<'ctx>(
    m: &Embedding<'ctx, 'ctx>,
    name: &str,
    bit: usize,
    input_fanout_by_name: &HashMap<String, Vec<(CellWrapper<'ctx>, usize)>>,
) -> Match<'ctx> {
    let pat = input_fanout_by_name.get(name).cloned().unwrap();
    let pat_first = &pat[0].0;
    let des = m.assignment.get_haystack_cell(pat_first.clone()).unwrap();
    Match {
        pat_node_ref: Some(pat_first.clone()),
        design_node_ref: Some(des),
    }
}

pub fn bind_output<'ctx>(
    m: &Embedding<'ctx, 'ctx>,
    name: &str,
    bit: usize,
    output_fanin_by_name: &HashMap<String, Vec<(CellWrapper<'ctx>, usize)>>,
) -> Match<'ctx> {
    let pat = output_fanin_by_name.get(name).cloned().unwrap();
    let pat_first = &pat[0].0;
    let des = m.assignment.get_haystack_cell(pat_first.clone()).unwrap();
    Match {
        pat_node_ref: Some(pat_first.clone()),
        design_node_ref: Some(des),
    }
}
