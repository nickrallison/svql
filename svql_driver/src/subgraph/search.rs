use crate::subgraph::cell_kind::CellWrapper;

use super::compat::cells_compatible;
use super::index::{Index, NodeId};
use super::ports::Source;
use super::state::State;
use super::{SubgraphMatch, cell_kind::{InputCell, OutputCell}};
use super::strategy::choose_next;

pub(super) fn backtrack<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &mut State<'p, 'd>,
    out: &mut Vec<SubgraphMatch<'p, 'd>>,
    pat_inputs: &[InputCell<'p>],
    pat_outputs: &[OutputCell<'p>],
) {
    if st.done() {
        out.push(st.to_subgraph_match(p_index, d_index, pat_inputs, pat_outputs));
        return;
    }

    let next_p = match choose_next(p_index, st) {
        Some(n) => n,
        None => return,
    };

    let kind = p_index.kind(next_p);
    for &d_cand in d_index.of_kind(kind) {
        if st.is_used_design(d_cand) {
            continue;
        }
        if !cells_compatible(next_p, d_cand, p_index, d_index, st) {
            continue;
        }

        st.map(next_p, d_cand);
        let added = add_io_boundaries_from_pair(next_p, d_cand, p_index, d_index, st);

        backtrack(p_index, d_index, st, out, pat_inputs, pat_outputs);

        remove_boundaries(added, st);
        st.unmap(next_p, d_cand);
    }
}

pub(super) fn add_io_boundaries_from_pair<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &mut State<'p, 'd>,
) -> Vec<(CellWrapper<'p>, usize)> {
    let mut added = Vec::new();
    let p_pins = &p_index.pins(p_id).inputs;
    let d_pins = &d_index.pins(d_id).inputs;

    for ((_, p_src), (_, d_src)) in p_pins.iter().zip(d_pins.iter()) {
        match (p_src, d_src) {
            (Source::Io(p_cell, p_bit), Source::Io(d_cell, d_bit)) => {
                let key = (*p_cell, *p_bit);
                if st.boundary_insert(key, (*d_cell, *d_bit)) {
                    added.push(key);
                }
            }
            (Source::Io(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                let key = (*p_cell, *p_bit);
                if st.boundary_insert(key, (*d_cell, *d_bit)) {
                    added.push(key);
                }
            }
            _ => {}
        }
    }

    added
}

pub(super) fn remove_boundaries<'p, 'd>(
    added: Vec<(CellWrapper<'p>, usize)>,
    st: &mut State<'p, 'd>,
) {
    st.boundary_remove_keys(&added);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{Driver};
    use crate::util::load_driver_from;

    lazy_static::lazy_static! {
        static ref SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/sdffe.v");
    }

    #[test]
    fn backtrack_self_sdffe_produces_some() {
        let d = SDFFE.0.design_as_ref();
        let p_index = Index::build(d);
        let d_index = Index::build(d);

        let mut st = State::new(p_index.gate_count());
        let mut out = Vec::new();
        let inputs = super::super::cell_kind::get_input_cells(d);
        let outputs = super::super::cell_kind::get_output_cells(d);

        backtrack(&p_index, &d_index, &mut st, &mut out, &inputs, &outputs);
        if !out.is_empty() {
            assert!(!out[0].is_empty());
        }
    }
}