use log::trace;

use crate::index::{Index, NodeId};
use crate::model::{
    CellKind,
    normalize::{is_commutative, normalize_commutative},
};

/// Compute deterministic aligned input pairs for pattern/design nodes,
/// respecting exact-length vs superset-length and commutativity normalization.
pub fn aligned_sources(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index,
    d_index: &Index,
    match_length: bool,
) -> Option<Vec<super::bindings::AlignedPair>> {
    trace!("Aligning sources for pattern {} and design {}", p_id, d_id);

    let kind: CellKind = p_index.kind(p_id);
    trace!("Cell kind: {:?}", kind);

    let mut p_inputs = p_index.pins(p_id).inputs.clone();
    let mut d_inputs = d_index.pins(d_id).inputs.clone();
    trace!(
        "Pattern inputs: {:?}, Design inputs: {:?}",
        p_inputs.len(),
        d_inputs.len()
    );

    if is_commutative(kind) {
        trace!("Normalizing commutative inputs");
        normalize_commutative(&mut p_inputs);
        normalize_commutative(&mut d_inputs);
        trace!(
            "After normalization - Pattern: {:?}, Design: {:?}",
            p_inputs, d_inputs
        );
    }

    let p_len = p_inputs.len();
    let d_len = d_inputs.len();
    trace!("Pattern len: {}, Design len: {}", p_len, d_len);

    if match_length && p_len != d_len {
        trace!("Length mismatch with exact matching - returning None");
        return None;
    }
    if !match_length && p_len > d_len {
        trace!("Pattern longer than design with superset matching - returning None");
        return None;
    }

    let take_len = std::cmp::min(p_len, d_len);
    trace!("Taking {} aligned pairs", take_len);

    let result: Vec<super::bindings::AlignedPair> = p_inputs
        .into_iter()
        .zip(d_inputs)
        .take(take_len)
        .map(|(pattern, design)| {
            trace!("Aligned pair: {:?} -> {:?}", pattern, design);
            super::bindings::AlignedPair { pattern, design }
        })
        .collect();

    trace!("Returning {} aligned pairs", result.len());
    Some(result)
}
