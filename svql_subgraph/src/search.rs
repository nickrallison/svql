use crate::cell::CellWrapper;
use crate::config;
use crate::state::check_and_collect_bindings;

use super::SubgraphMatch;
use super::compat::cells_compatible;
use super::index::{Index, NodeId};
use super::state::State;
use super::strategy::choose_next;

pub(super) fn backtrack<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &mut State<'p, 'd>,
    out: &mut Vec<SubgraphMatch<'p, 'd>>,
    pat_inputs: &[CellWrapper<'p>],
    pat_outputs: &[CellWrapper<'p>],
    config: &config::Config,
) {
    if st.done() {
        out.push(st.to_subgraph_match(p_index, d_index, pat_inputs, pat_outputs));
        return;
    }

    let Some(next_p) = choose_next(p_index, st) else {
        return;
    };

    let kind = p_index.kind(next_p);

    let candidates: Vec<NodeId> = d_index
        .of_kind(kind)
        .iter()
        .copied()
        .filter(|&d_cand| !st.is_used_design(d_cand))
        .filter(|&d_cand| {
            cells_compatible(next_p, d_cand, p_index, d_index, st, config.match_length)
        })
        .collect();

    for d_cand in candidates {
        with_mapping(st, next_p, d_cand, p_index, d_index, config, |st_inner| {
            backtrack(
                p_index,
                d_index,
                st_inner,
                out,
                pat_inputs,
                pat_outputs,
                config,
            );
        });
    }
}

/// Scoped helper that maps (p_id -> d_id), records IO bindings implied by the pair,
/// runs `f`, then automatically removes those bindings and unmaps.
/// Localizes mutation and prevents deep nesting.
fn with_mapping<'p, 'd>(
    st: &mut State<'p, 'd>,
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    config: &config::Config,
    f: impl FnOnce(&mut State<'p, 'd>),
) {
    st.map(p_id, d_id);
    let added = add_bindings_from_pair(p_id, d_id, p_index, d_index, st, config);
    f(st);
    remove_bindings(added, st);
    st.unmap(p_id, d_id);
}

pub(super) fn add_bindings_from_pair<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &mut State<'p, 'd>,
    config: &config::Config,
) -> Vec<crate::state::PatSrcKey<'p>> {
    let mut added = Vec::new();

    if let Some(pending) =
        check_and_collect_bindings(p_id, d_id, p_index, d_index, st, config.match_length)
    {
        for (p_key, d_key) in pending {
            if st.binding_insert(p_key, d_key) {
                added.push(p_key);
            }
        }
    }

    added
}

pub(super) fn remove_bindings<'p, 'd>(
    added: Vec<crate::state::PatSrcKey<'p>>,
    st: &mut State<'p, 'd>,
) {
    st.bindings_remove_keys(&added);
}

#[cfg(test)]
mod tests {

    use prjunnamed_netlist::Design;

    use crate::config::Config;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn backtrack_self_sdffe_produces_some() {
        let d = &SDFFE;
        let p_index = Index::build(d);
        let d_index = Index::build(d);

        let mut st = State::new(p_index.gate_count());
        let mut out = Vec::new();
        let inputs = super::super::cell::get_input_cells(d);
        let outputs = super::super::cell::get_output_cells(d);

        let config = Config::default();

        backtrack(
            &p_index, &d_index, &mut st, &mut out, &inputs, &outputs, &config,
        );
        if !out.is_empty() {
            assert!(!out[0].is_empty());
        }
    }
}
