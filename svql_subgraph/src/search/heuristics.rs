use crate::index::{Index, NodeId};
use crate::model::CellKind;

#[derive(Clone, Debug)]
pub(crate) struct ChosenCellSelection {
    pub pat_anchor: NodeId,
    pub des_anchors: Vec<NodeId>,
}

pub(crate) fn rarest_gate_heuristic<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
) -> Option<ChosenCellSelection> {
    // Build candidate triples (kind, d_count, p_count) only where pattern has that kind.
    let candidates: Vec<(CellKind, usize, usize)> = d_index
        .by_kind_iter()
        .into_iter()
        .filter_map(|kn| {
            let k = kn.0;
            let d_nodes_len = kn.1.len();
            let p_nodes = p_index.of_kind(k);
            (!p_nodes.is_empty()).then_some((k, d_nodes_len, p_nodes.len()))
        })
        .collect();

    // No candidates means no common kinds to anchor by.
    let (anchor_kind, _, _) =
        candidates
            .into_iter()
            .min_by(|(a_kind, a_d_count, _), (b_kind, b_d_count, _)| {
                // Primary: rarest in design
                let primary = a_d_count.cmp(b_d_count);
                if primary != std::cmp::Ordering::Equal {
                    return primary;
                }

                // Secondary: break tie by kind
                a_kind.cmp(b_kind)
            })?;

    // Deterministic order of anchors
    let mut p_anchors = p_index.of_kind(anchor_kind).to_vec();
    let mut d_anchors = d_index.of_kind(anchor_kind).to_vec();
    p_anchors.sort_unstable();
    d_anchors.sort_unstable();

    if p_anchors.is_empty() || d_anchors.is_empty() {
        return None;
    }
    Some(ChosenCellSelection {
        pat_anchor: p_anchors
            .first()
            .copied()
            .expect("No pattern anchors found"),
        des_anchors: d_anchors,
    })
}
