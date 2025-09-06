use crate::Match;
use svql_subgraph::SubgraphIsomorphism;

// Macro‑friendly helpers that bind a single bit of a named input or output
// into our `Match<'p, 'd>` payload. A codegen macro can call these for each
// declared port (and iterate width as needed if/when multi‑bit ports are
// supported at the query layer).

pub fn bind_input<'ctx>(
    m: &SubgraphIsomorphism<'ctx, 'ctx>,
    name: &str,
    bit: usize,
) -> Match<'ctx> {
    tracing::event!(tracing::Level::TRACE, "Binding input: {} bit {}", name, bit);
    let des = m.bound_inputs.get(name).cloned().unwrap();
    let pat = m.input_by_name.get(name).cloned().unwrap();
    Match {
        pat_node_ref: Some(pat),
        design_node_ref: Some(des),
    }
    // todo!()
}

pub fn bind_output<'ctx>(
    m: &SubgraphIsomorphism<'ctx, 'ctx>,
    name: &str,
    bit: usize,
) -> Match<'ctx> {
    tracing::event!(
        tracing::Level::TRACE,
        "Binding output: {} bit {}",
        name,
        bit
    );

    let des_res = m.bound_outputs.get(name).cloned();
    let des = match des_res {
        Some(v) => v,
        None => panic!("No bound output found for name {}", name),
    };
    if des.len() != 1 {
        panic!("For now, only single-bit outputs are supported");
    }
    let (des, _) = des.into_iter().next().unwrap();
    let pat = m.mapping.get_pattern_cell(des.clone()).unwrap();
    Match {
        pat_node_ref: Some(pat),
        design_node_ref: Some(des),
    }
    // todo!()
}
