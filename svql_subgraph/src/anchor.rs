use super::cell_kind::CellKind;
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
    // Gather candidate kinds only where both sides contain at least one gate of that kind.
    let mut candidates: Vec<(CellKind, usize, usize)> = Vec::new(); // (kind, d_count, p_count)

    // Deterministic iteration over kinds (Index::by_kind_iter is sorted).
    for (k_ref, d_nodes) in d_index.by_kind_iter() {
        let k = *k_ref;
        let p_nodes = p_index.of_kind(k);
        if p_nodes.is_empty() {
            continue;
        }
        candidates.push((k, d_nodes.len(), p_nodes.len()));
    }

    if candidates.is_empty() {
        return None;
    }

    // Primary: rarest in design.
    // Secondary: smallest design-to-pattern ratio.
    // Tertiary: deterministic tie-breaker by CellKind.
    candidates.sort_by(|a, b| {
        let primary = a.1.cmp(&b.1);
        if primary != std::cmp::Ordering::Equal {
            return primary;
        }

        // Compare a_ratio_num/a_ratio_den vs b_ratio_num/b_ratio_den without floating point.
        let a_ratio_num = a.1 as u64;
        let a_ratio_den = a.2 as u64;
        let b_ratio_num = b.1 as u64;
        let b_ratio_den = b.2 as u64;
        let secondary = (a_ratio_num * b_ratio_den).cmp(&(b_ratio_num * a_ratio_den));
        if secondary != std::cmp::Ordering::Equal {
            return secondary;
        }

        // Deterministic final tie-breaker
        a.0.cmp(&b.0)
    });

    let anchor_kind = candidates[0].0;

    let mut p_anchors = p_index.of_kind(anchor_kind).to_vec();
    let mut d_anchors = d_index.of_kind(anchor_kind).to_vec();

    // Ensure deterministic order of anchors
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
        static ref SDFFE: Design = load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn choose_anchors_some() {
        let d = &SDFFE;
        let p_index = super::Index::build(d);
        let d_index = super::Index::build(d);
        let chosen = choose_anchors(&p_index, &d_index).expect("should find anchors");
        assert_eq!(chosen.0, CellKind::Dff);
    }
}
