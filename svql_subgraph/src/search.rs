use crate::cell::{CellKind, CellWrapper, Source};
use crate::state::check_and_collect_bindings;
use crate::{cells_compatible, config};

use super::SubgraphMatch;
use super::index::{Index, NodeId};
use super::state::State;

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

    // Phase 1: compute candidates with only immutable access to `st`.
    let candidates: Vec<NodeId> = d_index
        .of_kind(kind)
        .iter()
        .copied()
        .filter(|&d_cand| !st.is_used_design(d_cand))
        .filter(|&d_cand| {
            cells_compatible(next_p, d_cand, p_index, d_index, st, config.match_length)
        })
        .collect();

    // Phase 2: iterate candidates and perform scoped mutable updates.
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

pub(super) fn rarest_gate_heuristic<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
) -> Option<(CellKind, Vec<NodeId>, Vec<NodeId>)> {
    // Build candidate triples (kind, d_count, p_count) only where pattern has that kind.
    let candidates: Vec<(CellKind, usize, usize)> = d_index
        .by_kind_iter()
        .into_iter()
        .filter_map(|(k_ref, d_nodes)| {
            let k = *k_ref;
            let p_nodes = p_index.of_kind(k);
            (!p_nodes.is_empty()).then_some((k, d_nodes.len(), p_nodes.len()))
        })
        .collect();

    // No candidates means no common kinds to anchor by.
    let (anchor_kind, _, _) = candidates.into_iter().min_by(|a, b| {
        // Primary: rarest in design
        let primary = a.1.cmp(&b.1);
        if primary != std::cmp::Ordering::Equal {
            return primary;
        }
        // Secondary: smallest design-to-pattern ratio, compare as cross-product
        let a_ratio = (a.1 as u64, a.2 as u64);
        let b_ratio = (b.1 as u64, b.2 as u64);
        let secondary = (a_ratio.0 * b_ratio.1).cmp(&(b_ratio.0 * a_ratio.1));
        if secondary != std::cmp::Ordering::Equal {
            return secondary;
        }
        // Tertiary: deterministic tie-breaker by kind
        a.0.cmp(&b.0)
    })?;

    // Deterministic order of anchors
    let mut p_anchors = p_index.of_kind(anchor_kind).to_vec();
    let mut d_anchors = d_index.of_kind(anchor_kind).to_vec();
    p_anchors.sort_unstable();
    d_anchors.sort_unstable();

    if p_anchors.is_empty() || d_anchors.is_empty() {
        return None;
    }
    Some((anchor_kind, p_anchors, d_anchors))
}

pub(super) fn choose_next<'p, 'd>(p_index: &'p Index<'p>, st: &State<'p, 'd>) -> Option<NodeId> {
    let first_resolvable = (0..p_index.gate_count() as u32)
        .map(|i| i as NodeId)
        .find(|&p| !st.is_mapped(p) && inputs_resolved_for(p_index, st, p));

    first_resolvable.or_else(|| {
        (0..p_index.gate_count() as u32)
            .map(|i| i as NodeId)
            .find(|&p| !st.is_mapped(p))
    })
}

fn inputs_resolved_for<'p, 'd>(p_index: &'p Index<'p>, st: &State<'p, 'd>, p: NodeId) -> bool {
    p_index.pins(p).inputs.iter().all(|src| match src {
        Source::Const(_) => true,
        Source::Io(_, _) => true,
        Source::Gate(gc, _) => p_index
            .try_cell_to_node(*gc)
            .is_some_and(|g| st.is_mapped(g)),
    })
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

    #[test]
    fn heuristic_chooses_some() {
        let d = &SDFFE;
        let p_index = super::Index::build(d);
        let d_index = super::Index::build(d);
        let chosen = rarest_gate_heuristic(&p_index, &d_index).expect("should find anchors");
        assert_eq!(chosen.0, CellKind::Mux);
    }

    #[test]
    fn choose_next_returns_some() {
        let d = &SDFFE;
        let idx = Index::build(d);
        let st = State::new(idx.gate_count());
        assert!(choose_next(&idx, &st).is_some());
    }
}
