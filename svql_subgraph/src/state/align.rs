use crate::index::{Index, NodeId};
use crate::model::{
    CellKind,
    normalize::{is_commutative, normalize_commutative},
};

/// Compute deterministic aligned input pairs for pattern/design nodes,
/// respecting exact-length vs superset-length and commutativity normalization.
pub(crate) fn aligned_sources<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    match_length: bool,
) -> Option<Vec<super::bindings::AlignedPair<'p, 'd>>> {
    let kind: CellKind = p_index.kind(p_id);

    let mut p_inputs = p_index.pins(p_id).inputs.clone();
    let mut d_inputs = d_index.pins(d_id).inputs.clone();

    if is_commutative(kind) {
        normalize_commutative(&mut p_inputs);
        normalize_commutative(&mut d_inputs);
    }

    let p_len = p_inputs.len();
    let d_len = d_inputs.len();

    if match_length && p_len != d_len {
        return None;
    }
    if !match_length && p_len > d_len {
        return None;
    }

    let take_len = std::cmp::min(p_len, d_len);

    Some(
        p_inputs
            .into_iter()
            .zip(d_inputs)
            .take(take_len)
            .map(|(pattern, design)| super::bindings::AlignedPair { pattern, design })
            .collect(),
    )
}
