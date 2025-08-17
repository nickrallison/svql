use std::collections::HashMap;

use prjunnamed_netlist::{Cell, Design};

pub mod cell_kind;
use cell_kind::{get_input_cells, get_output_cells, InputCell, OutputCell};

use crate::subgraph::cell_kind::CellWrapper;

mod ports;
mod index;
mod compat;
mod state;
mod strategy;
mod anchor;
mod search;

#[derive(Clone, Debug)]
pub struct AllSubgraphMatches<'p, 'd> {
    pub matches: Vec<SubgraphMatch<'p, 'd>>,
    pub _p_index: index::Index<'p>,
    pub _d_index: index::Index<'d>,
}

impl<'p, 'd> AllSubgraphMatches<'p, 'd> {
    pub fn len(&self) -> usize { self.matches.len() }
    pub fn is_empty(&self) -> bool { self.matches.is_empty() }
    pub fn iter(&self) -> std::slice::Iter<'_, SubgraphMatch<'p, 'd>> {
        self.matches.iter()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    pub cell_mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    pub pat_input_cells: Vec<InputCell<'p>>,
    pub pat_output_cells: Vec<OutputCell<'p>>,
    pub boundary_src_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)>,
}

impl<'p, 'd> SubgraphMatch<'p, 'd> {
    pub fn len(&self) -> usize { self.cell_mapping.len() }
    pub fn is_empty(&self) -> bool { self.cell_mapping.is_empty() }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, CellWrapper<'p>, CellWrapper<'d>> {
        self.cell_mapping.iter()
    }
}


// Public API
pub fn find_subgraphs<'p, 'd>(pattern: &'p Design, design: &'d Design) -> AllSubgraphMatches<'p, 'd> {
    let p_index = index::Index::build(pattern);
    let d_index = index::Index::build(design);

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        return AllSubgraphMatches {
            matches: Vec::new(),
            _p_index: p_index,
            _d_index: d_index,
        };
    }

    let Some((_anchor_kind, p_anchors, d_anchors)) = anchor::choose_anchors(&p_index, &d_index) else {
        return AllSubgraphMatches {
            matches: Vec::new(),
            _p_index: p_index,
            _d_index: d_index,
        };
    };

    let mut results: Vec<SubgraphMatch<'p, 'd>> = Vec::new();

    let (pat_inputs, pat_outputs) = get_pattern_io_cells(pattern);

    for &p_a in &p_anchors {
        for &d_a in &d_anchors {
        
            if p_index.kind(p_a) != d_index.kind(d_a) {
                continue;
            }
            // Quick compatibility check with empty state
            let empty_state = state::State::< 'p, 'd>::new(p_index.gate_count());
            if !compat::cells_compatible(p_a, d_a, &p_index, &d_index, &empty_state) {
                continue;
            }

            let mut st = state::State::new(p_index.gate_count());
            st.map(p_a, d_a);

            // Add IO boundaries implied by anchor mapping
            let added = search::add_io_boundaries_from_pair(p_a, d_a, &p_index, &d_index, &mut st);

            search::backtrack(&p_index, &d_index, &mut st, &mut results, &pat_inputs, &pat_outputs);

            // Backtrack anchor boundaries
            search::remove_boundaries(added, &mut st);
            st.unmap(p_a, d_a);
            
        }
    }

    AllSubgraphMatches {
        matches: results,
        _p_index: p_index,
        _d_index: d_index,
    }
}

// Helper used by tests and callers
pub fn get_pattern_io_cells<'p>(pattern: &'p Design) -> (Vec<InputCell<'p>>, Vec<OutputCell<'p>>) {
    (get_input_cells(pattern), get_output_cells(pattern))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{read_input_to_design, Driver};
    use prjunnamed_netlist::Design;
    use crate::util::load_driver_from;

    lazy_static::lazy_static! {
        static ref ASYNC_MUX: (Driver, PathBuf) = load_driver_from("examples/patterns/security/access_control/locked_reg/json/async_mux.json");
        static ref SEQ_DOUBLE_SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/seq_double_sdffe.v");
        static ref SDFFE: (Driver, PathBuf) = load_driver_from("examples/patterns/basic/ff/sdffe.v");
    }


    #[test]
    fn smoke_io_cells() {
        let design = ASYNC_MUX.0.design_as_ref();
        let (ins, outs) = get_pattern_io_cells(design);
        assert!(!ins.is_empty());
        assert!(!outs.is_empty());
    }

    #[test]
    fn smoke_find_subgraphs_self_sdffe() {
        let design = SDFFE.0.design_as_ref();
        let matches = find_subgraphs(design, design);
        assert!(!matches.is_empty(), "Self-match sdffe should yield at least one mapping");
        for m in matches.iter() {
            assert!(!m.is_empty());
        }
    }

    #[test]
    fn smoke_seq_double_sdffe_has_at_least_one() {
        let design = SEQ_DOUBLE_SDFFE.0.design_as_ref();
        let matches = find_subgraphs(design, design);
        assert!(!matches.is_empty(), "Self-match seq_double_sdffe should yield mappings");
    }
}