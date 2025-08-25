use crate::index::{Index, NodeId};
use crate::model::Source;

use super::State;

pub(crate) fn cells_compatible<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    state: &State<'p, 'd>,
    match_length: bool,
) -> bool {
    if p_index.kind(p_id) != d_index.kind(d_id) {
        return false;
    }

    super::bindings::check_and_collect_bindings(p_id, d_id, p_index, d_index, state, match_length)
        .is_some_and(|_| downstream_consumers_compatible(p_id, d_id, p_index, d_index, state))
}

/// Return all bit indices on q_p's inputs that are driven by p_id in the pattern.
#[contracts::debug_ensures(ret.iter().all(|&b| b < p_index.pins(q_p).inputs.len()))]
pub(crate) fn pattern_consumption_bits<'p>(
    p_index: &Index<'p>,
    q_p: NodeId,
    p_id: NodeId,
) -> Vec<usize> {
    p_index
        .pins(q_p)
        .inputs
        .iter()
        .filter_map(|p_src| {
            let p_src_cell_and_p_bit_opt = match p_src {
                Source::Gate(c, b) => Some((*c, *b)),
                _ => None,
            };
            
            p_src_cell_and_p_bit_opt
                .and_then(|(p_src_cell, p_bit)| {
                    let p_src_node = p_index.try_cell_to_node(p_src_cell)?;
                    (p_src_node == p_id).then_some(p_bit)
                })
        })
        .collect()
}

/// Does the mapped design consumer q_d have an input from d_id at the given bit?
pub(crate) fn design_has_input_from_bit<'d>(
    d_index: &Index<'d>,
    q_d: NodeId,
    d_id: NodeId,
    bit: usize,
) -> bool {
    d_index.pins(q_d).inputs.iter().any(|d_src| match d_src {
        Source::Gate(d_src_cell, d_bit) => d_index
            .try_cell_to_node(*d_src_cell)
            .is_some_and(|d_src_node| d_src_node == d_id && *d_bit == bit),
        _ => false,
    })
}

/// Ensure that for every already-mapped consumer (q_p -> q_d), any usage of p_id
/// as a source in q_p is mirrored by a usage of d_id in q_d at the same bit index.
pub(crate) fn downstream_consumers_compatible<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    state: &State<'p, 'd>,
) -> bool {
    state.mappings().iter().all(|(&q_p, &q_d)| {
        let required_bits = pattern_consumption_bits(p_index, q_p, p_id);
        required_bits
            .iter()
            .all(|&bit| design_has_input_from_bit(d_index, q_d, d_id, bit))
    })
}

/// Validate that a mapped gate pair is consistent if pattern node is already mapped.
/// If the pattern node is unmapped, any bit is acceptable at this stage.
pub(crate) fn mapped_gate_pair_ok<'p, 'd>(
    st: &State<'p, 'd>,
    p_node: NodeId,
    p_bit: usize,
    d_node: NodeId,
    d_bit: usize,
) -> bool {
    match st.mapped_to(p_node) {
        Some(mapped_d) => mapped_d == d_node && p_bit == d_bit,
        None => true,
    }
}
