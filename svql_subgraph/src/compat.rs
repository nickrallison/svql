use crate::state::check_and_collect_boundary;

use super::index::{Index, NodeId};
use super::ports::Source;
use super::state::State;

/// Check if two cells (pattern/design) are compatible under the current state and config.
pub(super) fn cells_compatible<'p, 'd>(
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

    // Pairwise pin checks + (side-effect-free) collection of boundary insertions
    let Some(_pending_boundary) =
        check_and_collect_boundary(p_id, d_id, p_index, d_index, state, match_length)
    else {
        return false;
    };

    downstream_consumers_compatible(p_id, d_id, p_index, d_index, state)
}

/// Return all bit indices on q_p's inputs that are driven by p_id in the pattern.
fn pattern_consumption_bits<'p>(p_index: &Index<'p>, q_p: NodeId, p_id: NodeId) -> Vec<usize> {
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
fn design_has_input_from_bit<'d>(
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
///
/// This enforces that mapping (p_id -> d_id) remains consistent with the portion
/// of the mapping already built, regardless of mapping order.
fn downstream_consumers_compatible<'p, 'd>(
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

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use crate::cell::CellKind;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v").unwrap();
    }

    #[test]
    fn same_cell_is_compatible_with_itself() {
        let d = &*SDFFE;
        let idx = super::Index::build(d);
        let st = super::State::<'_, '_>::new(idx.gate_count());
        let match_length = true;

        for &n in idx.of_kind(crate::cell::CellKind::Dff) {
            assert!(cells_compatible(n, n, &idx, &idx, &st, match_length));
        }
    }

    #[test]
    fn pattern_io_can_bind_to_design_gate() {
        let d_p = &SDFFE;
        let d_d = &SEQ_DOUBLE_SDFFE;

        let p_idx = super::Index::build(d_p);
        let d_idx = super::Index::build(d_d);
        let st = super::State::new(p_idx.gate_count());

        let match_length = true;

        let p = p_idx.of_kind(CellKind::Dff)[0];
        for &d in d_idx.of_kind(CellKind::Dff) {
            assert!(
                cells_compatible(p, d, &p_idx, &d_idx, &st, match_length),
                "pattern IO D should be compatible with design DFF regardless of external driver kind"
            );
        }
    }
}
