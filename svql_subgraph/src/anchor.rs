use super::cell::CellKind;
use super::index::{Index, NodeId};

/// Choose an anchor kind and the candidate anchors in both pattern and design.
///
/// Heuristic:
/// 1) Consider only kinds that exist in both pattern and design.
/// 2) Pick the rarest kind in the design (fewest instances).
/// 3) Break ties by the smallest (design_count / pattern_count) ratio.
/// 4) Final deterministic tie-breaker: by CellKind (Ord).
///
/// Returns:
/// - The chosen CellKind anchor
/// - All pattern node IDs of that kind (sorted)
/// - All design node IDs of that kind (sorted)
pub(super) fn choose_anchors<'p, 'd>(
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

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use crate::util::load_design_from;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn choose_anchors_some() {
        let d = &SDFFE;
        let p_index = super::Index::build(d);
        let d_index = super::Index::build(d);
        let chosen = choose_anchors(&p_index, &d_index).expect("should find anchors");
        assert_eq!(chosen.0, CellKind::Mux);
    }
}
