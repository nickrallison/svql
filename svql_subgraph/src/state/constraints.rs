use log::trace;

use crate::index::{Index, NodeId};
use crate::model::Source;

use super::State;

/// Return all bit indices on q_p's inputs that are driven by p_id in the pattern.
pub(crate) fn pattern_consumption_bits(p_index: &Index, q_p: NodeId, p_id: NodeId) -> Vec<usize> {
    p_index
        .pins(q_p)
        .inputs
        .iter()
        .filter_map(|p_src| {
            let (p_src_cell, p_bit) = match p_src {
                Source::Gate(c, b) => (*c, *b),
                _ => return None,
            };
            let p_src_node = p_index.try_cell_to_node(p_src_cell)?;
            (p_src_node == p_id).then_some(p_bit)
        })
        .collect()
}

/// Does the mapped design consumer q_d have an input from d_id at the given bit?
pub(crate) fn design_has_input_from_bit(
    d_index: &Index,
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

pub fn cells_compatible(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index,
    d_index: &Index,
    state: &State,
    match_length: bool,
) -> bool {
    trace!(
        "Checking compatibility for pattern {} and design {}",
        p_id, d_id
    );

    if p_index.kind(p_id) != d_index.kind(d_id) {
        trace!("Kind mismatch");
        return false;
    }

    let bindings_ok = super::bindings::check_and_collect_bindings(
        p_id,
        d_id,
        p_index,
        d_index,
        state,
        match_length,
    )
    .is_some();

    if !bindings_ok {
        trace!("Bindings check failed");
        return false;
    }

    let downstream_ok = downstream_consumers_compatible(p_id, d_id, p_index, d_index, state);
    trace!("Downstream consumers compatible: {}", downstream_ok);
    downstream_ok
}

pub fn downstream_consumers_compatible(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index,
    d_index: &Index,
    state: &State,
) -> bool {
    trace!(
        "Checking downstream consumers for pattern {} -> design {}",
        p_id, d_id
    );

    let result = state.mappings().iter().all(|(&q_p, &q_d)| {
        trace!("Checking mapping {} -> {}", q_p, q_d);
        let required_bits = pattern_consumption_bits(p_index, q_p, p_id);
        trace!("Required bits for pattern {}: {:?}", p_id, required_bits);

        let all_bits_ok = required_bits.iter().all(|&bit| {
            let ok = design_has_input_from_bit(d_index, q_d, d_id, bit);
            trace!("Design {} has input from {} bit {}: {}", q_d, d_id, bit, ok);
            ok
        });
        all_bits_ok
    });

    trace!("Downstream consumers compatible result: {}", result);
    result
}

/// Validate that a mapped gate pair is consistent if pattern node is already mapped.
/// If the pattern node is unmapped, any bit is acceptable at this stage.
pub(crate) fn mapped_gate_pair_ok(
    st: &State,
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
