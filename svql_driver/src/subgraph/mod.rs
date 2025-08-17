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
    pub p_index: index::Index<'p>,
    pub d_index: index::Index<'d>,
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
    let p_index: index::Index<'p> = index::Index::build(pattern);
    let d_index: index::Index<'d> = index::Index::build(design);

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        return AllSubgraphMatches {
            matches: Vec::new(),
            p_index,
            d_index,
        };
    }

    let Some((_anchor_kind, p_anchors, d_anchors)) = anchor::choose_anchors(&p_index, &d_index) else {
        return AllSubgraphMatches {
            matches: Vec::new(),
            p_index,
            d_index,
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
        p_index,
        d_index,
    }
}

// Helper used by tests and callers
pub fn get_pattern_io_cells<'p>(pattern: &'p Design) -> (Vec<InputCell<'p>>, Vec<OutputCell<'p>>) {
    (get_input_cells(pattern), get_output_cells(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_input_to_design;

    lazy_static::lazy_static! {
        static ref ASYNC_MUX: Design = load_design_from("examples/patterns/security/access_control/locked_reg/json/async_mux.json");
        static ref DOUBLE_SDFFE: Design = load_design_from("examples/patterns/basic/ff/double_sdffe.v");
        static ref SDFFE: Design = load_design_from("examples/patterns/basic/ff/sdffe.v");
    }

    fn load_design_from(path: &str) -> Design {
        read_input_to_design(None, path.to_string()).expect("Failed to read input design")
    }

    #[test]
    fn smoke_io_cells() {
        let design = &*ASYNC_MUX;
        let (ins, outs) = get_pattern_io_cells(design);
        assert!(!ins.is_empty());
        assert!(!outs.is_empty());
    }

    #[test]
    fn smoke_find_subgraphs_self_sdffe() {
        let design = &*SDFFE;
        let matches = find_subgraphs(design, design);
        assert!(!matches.is_empty(), "Self-match sdffe should yield at least one mapping");
        for m in matches {
            assert!(!m.is_empty());
        }
    }

    #[test]
    fn smoke_double_sdffe_has_at_least_one() {
        let design = &*DOUBLE_SDFFE;
        let matches = find_subgraphs(design, design);
        assert!(!matches.is_empty(), "Self-match double_sdffe should yield mappings");
    }
}