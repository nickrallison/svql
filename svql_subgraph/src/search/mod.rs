use crate::index::{Index, NodeId};
use crate::model::{CellWrapper, Source};
// use crate::state::{PatSrcKey, State, check_and_collect_bindings};
use svql_common::Config;

use super::SubgraphMatch;

pub mod heuristics;
pub(crate) use heuristics::rarest_gate_heuristic;

// pub(super) fn backtrack<'p, 'd>(
//     p_index: &Index<'p>,
//     d_index: &Index<'d>,
//     st: &mut State<'p, 'd>,
//     out: &mut Vec<SubgraphMatch<'p, 'd>>,
//     pat_inputs: &[CellWrapper<'p>],
//     pat_outputs: &[CellWrapper<'p>],
//     config: &Config,
// ) {
//     if st.done() {
//         out.push(st.to_subgraph_match(p_index, d_index, pat_inputs, pat_outputs));
//         return;
//     }

//     let Some(next_p) = choose_next(p_index, st) else {
//         return;
//     };

//     let kind = p_index.kind(next_p);

//     // Phase 1: compute candidates with only immutable access to `st`.
//     let candidates: Vec<NodeId> = d_index
//         .of_kind(kind)
//         .iter()
//         .copied()
//         .filter(|&d_cand| !st.is_used_design(d_cand))
//         .filter(|&d_cand| {
//             crate::state::cells_compatible(
//                 next_p,
//                 d_cand,
//                 p_index,
//                 d_index,
//                 st,
//                 config.match_length,
//             )
//         })
//         .collect();

//     // Phase 2: iterate candidates and perform scoped mutable updates.
//     for d_cand in candidates {
//         with_mapping(st, next_p, d_cand, p_index, d_index, config, |st_inner| {
//             backtrack(
//                 p_index,
//                 d_index,
//                 st_inner,
//                 out,
//                 pat_inputs,
//                 pat_outputs,
//                 config,
//             );
//         });
//     }
// }

// /// Scoped helper that maps (p_id -> d_id), records IO bindings implied by the pair,
// /// runs `f`, then automatically removes those bindings and unmaps.
// #[contracts::debug_requires(!st.is_mapped(p_id), "pattern id must be unmapped")]
// #[contracts::debug_requires(!st.is_used_design(d_id), "design id must be unused")]
// #[contracts::debug_ensures(!st.is_mapped(p_id) && !st.is_used_design(d_id), "mapping removed")]
// fn with_mapping<'p, 'd>(
//     st: &mut State<'p, 'd>,
//     p_id: NodeId,
//     d_id: NodeId,
//     p_index: &Index<'p>,
//     d_index: &Index<'d>,
//     config: &Config,
//     f: impl FnOnce(&mut State<'p, 'd>),
// ) {
//     st.map(p_id, d_id);
//     let added = add_bindings_from_pair(p_id, d_id, p_index, d_index, st, config);
//     f(st);
//     remove_bindings(added, st);
//     st.unmap(p_id, d_id);
// }

// #[contracts::debug_ensures(ret.iter().all(|k| st.binding_get(*k).is_some()))]
// pub(super) fn add_bindings_from_pair<'p, 'd>(
//     p_id: NodeId,
//     d_id: NodeId,
//     p_index: &Index<'p>,
//     d_index: &Index<'d>,
//     st: &mut State<'p, 'd>,
//     config: &Config,
// ) -> Vec<PatSrcKey<'p>> {
//     let mut added = Vec::new();

//     if let Some(pending) =
//         check_and_collect_bindings(p_id, d_id, p_index, d_index, st, config.match_length)
//     {
//         for add in pending {
//             if st.binding_insert(add.pattern, add.design) {
//                 added.push(add.pattern);
//             }
//         }
//     }

//     added
// }

// pub(super) fn remove_bindings<'p, 'd>(added: Vec<PatSrcKey<'p>>, st: &mut State<'p, 'd>) {
//     st.bindings_remove_keys(&added);
// }

// pub(super) fn choose_next<'p, 'd>(p_index: &'p Index<'p>, st: &State<'p, 'd>) -> Option<NodeId> {
//     let first_resolvable = (0..p_index.gate_count() as u32)
//         .map(|i| i as NodeId)
//         .find(|&p| !st.is_mapped(p) && inputs_resolved_for(p_index, st, p));

//     first_resolvable.or_else(|| {
//         (0..p_index.gate_count() as u32)
//             .map(|i| i as NodeId)
//             .find(|&p| !st.is_mapped(p))
//     })
// }

// fn inputs_resolved_for<'p, 'd>(p_index: &'p Index<'p>, st: &State<'p, 'd>, p: NodeId) -> bool {
//     p_index.pins(p).inputs.iter().all(|src| match src {
//         Source::Const(_) => true,
//         Source::Io(_, _) => true,
//         Source::Gate(gc, _) => p_index
//             .try_cell_to_node(*gc)
//             .is_some_and(|g| st.is_mapped(g)),
//     })
// }
