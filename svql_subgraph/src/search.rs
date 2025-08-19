use crate::cell_kind::CellWrapper;
use crate::config;
use crate::ports::{is_commutative, normalize_commutative};

use super::compat::cells_compatible;
use super::index::{Index, NodeId};
use super::ports::Source;
use super::state::State;
use super::strategy::choose_next;
use super::{
    SubgraphMatch,
    cell_kind::{InputCell, OutputCell},
};

pub(super) fn backtrack<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &mut State<'p, 'd>,
    out: &mut Vec<SubgraphMatch<'p, 'd>>,
    pat_inputs: &[InputCell<'p>],
    pat_outputs: &[OutputCell<'p>],
    config: &config::Config,
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
        if !cells_compatible(next_p, d_cand, p_index, d_index, st, config.match_length) {
            continue;
        }

        st.map(next_p, d_cand);
        let added = add_io_boundaries_from_pair(next_p, d_cand, p_index, d_index, st, config);

        backtrack(p_index, d_index, st, out, pat_inputs, pat_outputs, config);

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
    config: &config::Config,
) -> Vec<(CellWrapper<'p>, usize)> {
    let mut added = Vec::new();

    let kind = p_index.kind(p_id);
    let mut p_inputs = p_index.pins(p_id).inputs.clone();
    let mut d_inputs = d_index.pins(d_id).inputs.clone();

    if is_commutative(kind) {
        normalize_commutative(&mut p_inputs);
        normalize_commutative(&mut d_inputs);
    }

    let p_len = p_inputs.len();
    let d_len = d_inputs.len();

    let take_len = std::cmp::min(p_len, d_len);

    for i in 0..take_len {
        let (_, p_src) = p_inputs[i];
        let (_, d_src) = d_inputs[i];

        match (p_src, d_src) {
            (Source::Io(p_cell, p_bit), Source::Io(d_cell, d_bit)) => {
                let key = (p_cell, p_bit);
                if st.boundary_insert(key, (d_cell, d_bit)) {
                    added.push(key);
                }
            }
            (Source::Io(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                let key = (p_cell, p_bit);
                if st.boundary_insert(key, (d_cell, d_bit)) {
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

    use prjunnamed_netlist::Design;

    use crate::config::Config;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn backtrack_self_sdffe_produces_some() {
        let d = &SDFFE;
        let p_index = Index::build(d);
        let d_index = Index::build(d);

        let mut st = State::new(p_index.gate_count());
        let mut out = Vec::new();
        let inputs = super::super::cell_kind::get_input_cells(d);
        let outputs = super::super::cell_kind::get_output_cells(d);

        let config = Config::default();

        backtrack(
            &p_index, &d_index, &mut st, &mut out, &inputs, &outputs, &config,
        );
        if !out.is_empty() {
            assert!(!out[0].is_empty());
        }
    }
}
