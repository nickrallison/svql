use super::cell_kind::CellKind;
use super::index::{Index, NodeId};

pub(super) fn choose_anchors<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
) -> Option<(CellKind, Vec<NodeId>, Vec<NodeId>)> {
    // Count kinds in design
    let mut design_counts = Vec::new();
    for (&kind, nodes) in d_index_kind_iter(d_index) {
        design_counts.push((kind, nodes.len()));
    }

    // Candidate kinds: present in both pattern and design
    let mut candidates: Vec<(CellKind, usize)> = design_counts
        .into_iter()
        .filter(|(k, _)| !p_index.of_kind(*k).is_empty())
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Pick the rarest kind in the design
    candidates.sort_by(|a, b| a.1.cmp(&b.1));
    let anchor_kind = candidates[0].0;

    let p_anchors = p_index.of_kind(anchor_kind).to_vec();
    let d_anchors = d_index.of_kind(anchor_kind).to_vec();

    if p_anchors.is_empty() || d_anchors.is_empty() {
        return None;
    }

    Some((anchor_kind, p_anchors, d_anchors))
}

fn d_index_kind_iter<'a>(
    d_index: &'a Index<'a>,
) -> impl Iterator<Item = (&'a CellKind, &'a [super::index::NodeId])> {
    // Build a slice of tuples for iteration
    let mut kinds = Vec::new();
    for k in all_gate_kinds() {
        let nodes = d_index.of_kind(*k);
        if !nodes.is_empty() {
            kinds.push((k, nodes));
        }
    }
    kinds.into_iter()
}

fn all_gate_kinds() -> &'static [CellKind] {
    use CellKind::*;
    &[
        Buf, Not, And, Or, Xor, Mux, Adc, Aig, Eq, ULt, SLt, Shl, UShr, SShr, XShr, Mul, UDiv,
        UMod, SDivTrunc, SDivFloor, SModTrunc, SModFloor, Dff,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Driver;
    use crate::util::load_driver_from;

    lazy_static::lazy_static! {
        static ref SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn choose_anchors_some() {
        let d = &*SDFFE;
        let p_index = super::Index::build(d.design_as_ref());
        let d_index = super::Index::build(d.design_as_ref());
        let chosen = choose_anchors(&p_index, &d_index);
        assert!(chosen.is_some());
    }
}
