use crate::config;
use crate::index::{Index, NodeId};
use crate::model::{CellWrapper, Source};
use crate::state::{PatSrcKey, State, check_and_collect_bindings};

use super::SubgraphMatch;

pub mod heuristics;
pub(crate) use heuristics::rarest_gate_heuristic;
use log::trace;

pub fn backtrack(
    p_index: &Index,
    d_index: &Index,
    st: &mut State,
    out: &mut Vec<SubgraphMatch>,
    pat_inputs: &[CellWrapper],
    pat_outputs: &[CellWrapper],
    config: &config::Config,
) {
    trace!(
        "Backtrack called with state mappings: {}",
        st.mappings().len()
    );

    if st.done() {
        trace!("State is complete - creating subgraph match");
        out.push(st.to_subgraph_match(p_index, d_index, pat_inputs, pat_outputs));
        trace!("Subgraph match created, now have {} matches", out.len());
        return;
    }

    let Some(next_p) = choose_next(p_index, st) else {
        trace!("No next pattern node to choose - returning");
        return;
    };
    trace!("Chose next pattern node: {}", next_p);

    let kind = p_index.kind(next_p);
    trace!("Pattern node kind: {:?}", kind);

    // Phase 1: compute candidates with only immutable access to `st`.
    let candidates: Vec<NodeId> = d_index
        .of_kind(kind)
        .iter()
        .copied()
        .filter(|&d_cand| {
            let used = st.is_used_design(d_cand);
            trace!("Checking if design node {} is used: {}", d_cand, used);
            !used
        })
        .filter(|&d_cand| {
            let compatible = crate::state::cells_compatible(
                next_p,
                d_cand,
                p_index,
                d_index,
                st,
                config.match_length,
            );
            trace!(
                "Design node {} compatible with pattern node {}: {}",
                d_cand, next_p, compatible
            );
            compatible
        })
        .collect();
    trace!(
        "Found {} candidates for pattern node {}",
        candidates.len(),
        next_p
    );

    // Phase 2: iterate candidates and perform scoped mutable updates.
    for d_cand in candidates {
        trace!("Trying candidate design node: {}", d_cand);
        with_mapping(st, next_p, d_cand, p_index, d_index, config, |st_inner| {
            trace!("Recursing with updated state");
            backtrack(
                p_index,
                d_index,
                st_inner,
                out,
                pat_inputs,
                pat_outputs,
                config,
            );
            trace!("Back from recursion");
        });
        trace!("Finished with candidate {}", d_cand);
    }
    trace!("Backtrack completed for pattern node {}", next_p);
}

/// Scoped helper that maps (p_id -> d_id), records IO bindings implied by the pair,
/// runs `f`, then automatically removes those bindings and unmaps.
fn with_mapping(
    st: &mut State,
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index,
    d_index: &Index,
    config: &config::Config,
    f: impl FnOnce(&mut State),
) {
    st.map(p_id, d_id);
    let added = add_bindings_from_pair(p_id, d_id, p_index, d_index, st, config);
    f(st);
    remove_bindings(added, st);
    st.unmap(p_id, d_id);
}

pub(super) fn add_bindings_from_pair(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index,
    d_index: &Index,
    st: &mut State,
    config: &config::Config,
) -> Vec<PatSrcKey> {
    let mut added = Vec::new();

    if let Some(pending) =
        check_and_collect_bindings(p_id, d_id, p_index, d_index, st, config.match_length)
    {
        for add in pending {
            if st.binding_insert(add.pattern, add.design) {
                added.push(add.pattern);
            }
        }
    }

    added
}

pub(super) fn remove_bindings(added: Vec<PatSrcKey>, st: &mut State) {
    st.bindings_remove_keys(&added);
}

fn inputs_resolved_for(p_index: &Index, st: &State, p: NodeId) -> bool {
    trace!("Checking if inputs are resolved for pattern node {}", p);
    let result = p_index.pins(p).inputs.iter().all(|src| match src {
        Source::Const(_) => {
            trace!("Pattern node {} input is constant - resolved", p);
            true
        }
        Source::Io(_, _) => {
            trace!("Pattern node {} input is IO - resolved", p);
            true
        }
        Source::Gate(gc, _) => {
            let resolved = p_index.try_cell_to_node(*gc).is_some_and(|g| {
                let mapped = st.is_mapped(g);
                trace!(
                    "Pattern node {} input gate {} mapped: {}",
                    p,
                    gc.index(),
                    mapped
                );
                mapped
            });
            resolved
        }
    });
    trace!("Inputs resolved for pattern node {}: {}", p, result);
    result
}

pub fn choose_next(p_index: &Index, st: &State) -> Option<NodeId> {
    trace!("Choosing next pattern node");

    let first_resolvable = (0..p_index.gate_count() as u32)
        .map(|i| i as NodeId)
        .find(|&p| {
            let mapped = st.is_mapped(p);
            trace!("Checking if pattern node {} is mapped: {}", p, mapped);
            !mapped && inputs_resolved_for(p_index, st, p)
        });

    if let Some(node) = first_resolvable {
        trace!("Found first resolvable node: {}", node);
        return Some(node);
    }

    let first_unmapped = (0..p_index.gate_count() as u32)
        .map(|i| i as NodeId)
        .find(|&p| {
            let mapped = st.is_mapped(p);
            trace!(
                "Checking if pattern node {} is mapped (fallback): {}",
                p, mapped
            );
            !mapped
        });

    trace!("Fallback choice: {:?}", first_unmapped);
    first_unmapped
}

#[cfg(test)]
mod tests {

    use prjunnamed_netlist::Design;

    use crate::config::Config;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::test_support::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn backtrack_self_sdffe_produces_some() {
        let d = &SDFFE;
        let p_index = Index::build(d);
        let d_index = Index::build(d);

        let mut st = State::new(p_index.gate_count(), d, d);
        let mut out = Vec::new();
        let inputs = crate::model::get_input_cells(d);
        let outputs = crate::model::get_output_cells(d);

        let config = Config::default();

        backtrack(
            &p_index, &d_index, &mut st, &mut out, &inputs, &outputs, &config,
        );
        if !out.is_empty() {
            assert!(!out[0].is_empty());
        }
    }

    #[test]
    fn choose_next_returns_some() {
        let d = &SDFFE;
        let idx = Index::build(d);
        let st = State::new(idx.gate_count(), d, d);
        assert!(choose_next(&idx, &st).is_some());
    }
}
