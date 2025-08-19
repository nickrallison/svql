use super::index::{Index, NodeId};
use super::ports::{Source, is_commutative, normalize_commutative};
use super::state::State;

pub(super) fn cells_compatible<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    state: &State<'p, 'd>,
    match_length: bool,
) -> bool {
    let pk = p_index.kind(p_id);
    let dk = d_index.kind(d_id);
    if pk != dk {
        return false;
    }

    // if !match_length {
    //     return true;
    // }

    let p_pins = &p_index.pins(p_id).inputs;
    let d_pins = &d_index.pins(d_id).inputs;
    if p_pins.len() != d_pins.len() {
        return false;
    }

    if is_commutative(pk) {
        let mut p_sorted = p_pins.clone();
        let mut d_sorted = d_pins.clone();
        normalize_commutative(&mut p_sorted);
        normalize_commutative(&mut d_sorted);
        pins_compatible_pairwise(&p_sorted, &d_sorted, p_index, d_index, state)
    } else {
        pins_compatible_pairwise(p_pins, d_pins, p_index, d_index, state)
    }
}

fn pins_compatible_pairwise<'p, 'd>(
    p_pins: &[(super::ports::PinKind, Source<'p>)],
    d_pins: &[(super::ports::PinKind, Source<'d>)],
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    state: &State<'p, 'd>,
) -> bool {
    for ((_, p_src), (_, d_src)) in p_pins.iter().zip(d_pins.iter()) {
        match (p_src, d_src) {
            (Source::Const(pc), Source::Const(dc)) => {
                if pc != dc {
                    return false;
                }
            }
            (Source::Gate(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                // If the source gate in pattern is already mapped, enforce it matches.
                if let Some(p_node) = p_index.try_cell_to_node(*p_cell)
                    && let Some(mapped_d_node) = state.mapped_to(p_node)
                {
                    if let Some(d_node) = d_index.try_cell_to_node(*d_cell) {
                        if mapped_d_node != d_node || p_bit != d_bit {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
            (Source::Io(p_cell, p_bit), Source::Io(d_cell, d_bit)) => {
                if let Some((exp_d_cell, exp_d_bit)) = state.boundary_get(*p_cell, *p_bit)
                    && (exp_d_cell != *d_cell || exp_d_bit != *d_bit)
                {
                    return false;
                }
            }
            (Source::Io(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                if let Some((exp_d_cell, exp_d_bit)) = state.boundary_get(*p_cell, *p_bit)
                    && (exp_d_cell != *d_cell || exp_d_bit != *d_bit)
                {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/seq_double_sdffe.v").unwrap();
    }

    #[test]
    fn same_cell_kind_is_compatible_with_itself() {
        let d = &*SDFFE;
        let idx = Index::build(d);
        let st = State::<'_, '_>::new(idx.gate_count());
        let match_length = true;

        for &n in idx.of_kind(crate::cell_kind::CellKind::Dff) {
            assert!(cells_compatible(n, n, &idx, &idx, &st, match_length));
        }
    }

    #[test]
    fn pattern_io_can_bind_to_design_gate() {
        let d_p = &SDFFE;
        let d_d = &SEQ_DOUBLE_SDFFE;

        let p_idx = Index::build(d_p);
        let d_idx = Index::build(d_d);
        let st = State::new(p_idx.gate_count());

        let match_length = true;

        let p = p_idx.of_kind(crate::cell_kind::CellKind::Dff)[0];
        for &d in d_idx.of_kind(crate::cell_kind::CellKind::Dff) {
            assert!(
                cells_compatible(p, d, &p_idx, &d_idx, &st, match_length),
                "pattern IO D should be compatible with design DFF regardless of external driver kind"
            );
        }
    }
}
