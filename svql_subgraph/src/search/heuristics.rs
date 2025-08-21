use log::trace;

use crate::index::{Index, NodeId};
use crate::model::CellKind;

#[derive(Clone, Debug)]
pub(crate) struct ChosenCellSelection {
    pub pat_anchors: Vec<NodeId>,
    pub des_anchors: Vec<NodeId>,
}

pub fn rarest_gate_heuristic<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
) -> Option<ChosenCellSelection> {
    trace!("Running rarest gate heuristic");

    // Build candidate triples (kind, d_count, p_count) only where pattern has that kind.
    let candidates: Vec<(CellKind, usize, usize)> = d_index
        .by_kind_iter()
        .into_iter()
        .filter_map(|kn| {
            let k = kn.kind;
            let d_nodes_len = kn.nodes.len();
            let p_nodes = p_index.of_kind(k);
            let result = (!p_nodes.is_empty()).then_some((k, d_nodes_len, p_nodes.len()));
            if let Some((kind, d_count, p_count)) = result {
                trace!(
                    "Candidate kind: {:?}, design count: {}, pattern count: {}",
                    kind, d_count, p_count
                );
            }
            result
        })
        .collect();
    trace!("Found {} candidates", candidates.len());

    // No candidates means no common kinds to anchor by.
    let (anchor_kind, _, _) = candidates.into_iter().min_by(|a, b| {
        // Primary: rarest in design
        let primary = a.1.cmp(&b.1);
        if primary != std::cmp::Ordering::Equal {
            trace!(
                "Primary comparison: {:?} vs {:?} -> {:?}",
                a.0, b.0, primary
            );
            return primary;
        }
        // Secondary: smallest design-to-pattern ratio, compare as cross-product
        let a_ratio = (a.1 as u64, a.2 as u64);
        let b_ratio = (b.1 as u64, b.2 as u64);
        let secondary = (a_ratio.0 * b_ratio.1).cmp(&(b_ratio.0 * a_ratio.1));
        if secondary != std::cmp::Ordering::Equal {
            trace!(
                "Secondary comparison: {:?} vs {:?} -> {:?}",
                a.0, b.0, secondary
            );
            return secondary;
        }
        // Tertiary: deterministic tie-breaker by kind
        let tertiary = a.0.cmp(&b.0);
        trace!(
            "Tertiary comparison: {:?} vs {:?} -> {:?}",
            a.0, b.0, tertiary
        );
        tertiary
    })?;

    trace!("Selected anchor kind: {:?}", anchor_kind);

    // Deterministic order of anchors
    let mut p_anchors = p_index.of_kind(anchor_kind).to_vec();
    let mut d_anchors = d_index.of_kind(anchor_kind).to_vec();
    trace!("Pattern anchors before sort: {:?}", p_anchors);
    trace!("Design anchors before sort: {:?}", d_anchors);

    p_anchors.sort_unstable();
    d_anchors.sort_unstable();

    trace!("Pattern anchors after sort: {:?}", p_anchors);
    trace!("Design anchors after sort: {:?}", d_anchors);

    if p_anchors.is_empty() || d_anchors.is_empty() {
        trace!("No anchors found for selected kind");
        return None;
    }

    trace!(
        "Returning {} pattern anchors and {} design anchors",
        p_anchors.len(),
        d_anchors.len()
    );

    Some(ChosenCellSelection {
        pat_anchors: p_anchors,
        des_anchors: d_anchors,
    })
}
