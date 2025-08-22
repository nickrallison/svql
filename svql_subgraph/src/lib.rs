use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Design;

mod dedupe;
mod index;
mod model;
mod search;
mod state;

// Backward-compatible re-export for existing tests and users.
#[doc(hidden)]
pub mod util {
    pub use crate::test_support::load_design_from;
}

// Test support module (not cfg-gated to keep integration tests working).
pub mod test_support;

pub use model::CellWrapper;
use svql_common::{Config, DedupeMode};

use crate::dedupe::{signature_mapped_gate_set, signature_with_boundary};
use crate::index::Index;
use crate::model::{get_input_cells, get_output_cells};
use crate::search::heuristics::ChosenCellSelection;
use crate::search::{backtrack, rarest_gate_heuristic};
use crate::state::{State, cells_compatible};

#[derive(Clone, Debug)]
pub struct AllSubgraphMatches<'p, 'd> {
    pub matches: Vec<SubgraphMatch<'p, 'd>>,
}

impl<'p, 'd> AllSubgraphMatches<'p, 'd> {
    pub fn len(&self) -> usize {
        self.matches.len()
    }
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, SubgraphMatch<'p, 'd>> {
        self.matches.iter()
    }
    pub fn first(&self) -> Option<&SubgraphMatch<'p, 'd>> {
        self.matches.first()
    }
}

impl<'p, 'd> IntoIterator for AllSubgraphMatches<'p, 'd> {
    type Item = SubgraphMatch<'p, 'd>;
    type IntoIter = std::vec::IntoIter<SubgraphMatch<'p, 'd>>;
    fn into_iter(self) -> Self::IntoIter {
        self.matches.into_iter()
    }
}

impl<'p, 'd, 'a> IntoIterator for &'a AllSubgraphMatches<'p, 'd> {
    type Item = &'a SubgraphMatch<'p, 'd>;
    type IntoIter = std::slice::Iter<'a, SubgraphMatch<'p, 'd>>;
    fn into_iter(self) -> Self::IntoIter {
        self.matches.iter()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    pub cell_mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    pub pat_input_cells: Vec<CellWrapper<'p>>,
    pub pat_output_cells: Vec<CellWrapper<'p>>,
    pub boundary_src_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)>,

    pub input_by_name: HashMap<&'p str, CellWrapper<'p>>,
    pub output_by_name: HashMap<&'p str, CellWrapper<'p>>,
    pub out_driver_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)>,
}

impl<'p, 'd> SubgraphMatch<'p, 'd> {
    pub fn len(&self) -> usize {
        self.cell_mapping.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cell_mapping.is_empty()
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, CellWrapper<'p>, CellWrapper<'d>> {
        self.cell_mapping.iter()
    }

    pub fn design_source_of_input_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        let p_in = *self.input_by_name.get(name)?;
        self.boundary_src_map.get(&(p_in, bit)).copied()
    }

    pub fn design_driver_of_output_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        let p_out = *self.output_by_name.get(name)?;
        self.out_driver_map.get(&(p_out, bit)).copied()
    }
}

/// Original public API (borrowed references).
pub fn find_subgraphs<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> AllSubgraphMatches<'p, 'd> {
    let p_index = Index::build(pattern);
    let d_index = Index::build(design);

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        return AllSubgraphMatches {
            matches: Vec::new(),
        };
    }

    let Some(anchor @ ChosenCellSelection { .. }) = rarest_gate_heuristic(&p_index, &d_index)
    else {
        return AllSubgraphMatches {
            matches: Vec::new(),
        };
    };

    let mut results: Vec<SubgraphMatch<'p, 'd>> = Vec::new();
    let (pat_inputs, pat_outputs) = get_pattern_io_cells(pattern);

    // Deterministically pick a single pattern anchor (the minimum NodeId).
    let p_a = *anchor
        .pat_anchors
        .iter()
        .min()
        .expect("No pattern anchors found");

    for &d_a in &anchor.des_anchors {
        let empty_state = State::<'p, 'd>::new(p_index.gate_count());
        if !cells_compatible(
            p_a,
            d_a,
            &p_index,
            &d_index,
            &empty_state,
            config.match_length,
        ) {
            continue;
        }

        let mut st = State::new(p_index.gate_count());
        st.map(p_a, d_a);
        let added = search::add_bindings_from_pair(p_a, d_a, &p_index, &d_index, &mut st, config);

        backtrack(
            &p_index,
            &d_index,
            &mut st,
            &mut results,
            &pat_inputs,
            &pat_outputs,
            config,
        );

        search::remove_bindings(added, &mut st);
        st.unmap(p_a, d_a);
    }

    match config.dedupe {
        DedupeMode::None => {
            let mut seen = HashSet::new();
            results.retain(|m| seen.insert(signature_with_boundary(m)));
        }
        DedupeMode::AutoMorph => {
            let mut seen = std::collections::HashSet::new();
            results.retain(|m| seen.insert(signature_mapped_gate_set(m)));
        }
    }

    AllSubgraphMatches { matches: results }
}

// Internal helper for tests and internal wiring only.
pub(crate) fn get_pattern_io_cells<'p>(
    pattern: &'p Design,
) -> (Vec<CellWrapper<'p>>, Vec<CellWrapper<'p>>) {
    (get_input_cells(pattern), get_output_cells(pattern))
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use svql_common::{Config, ConfigBuilder, DedupeMode};

    use super::*;

    lazy_static::lazy_static! {
        static ref ASYNC_MUX: Design = crate::test_support::load_design_from("examples/patterns/security/access_control/locked_reg/json/async_mux.json").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::test_support::load_design_from("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v").unwrap();
        static ref SDFFE: Design = crate::test_support::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
        static ref COMB_D_DOUBLE_SDFFE: Design = crate::test_support::load_design_from("examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v").unwrap();
        static ref PAR_DOUBLE_SDFFE: Design = crate::test_support::load_design_from("examples/fixtures/basic/ff/verilog/par_double_sdffe.v").unwrap();
    }

    #[test]
    fn smoke_io_cells() {
        let design = &ASYNC_MUX;
        let (ins, outs) = get_pattern_io_cells(design);
        assert!(!ins.is_empty());
        assert!(!outs.is_empty());
    }

    #[test]
    fn smoke_find_subgraphs_self_sdffe() {
        let design = &SDFFE;
        let config = Config::default();
        let matches = find_subgraphs(design, design, &config);
        assert!(!matches.is_empty());
        for m in matches.iter() {
            assert!(!m.is_empty());
        }
    }

    #[test]
    fn dedupe_modes_do_not_panic() {
        let pat = &SDFFE;
        let hay = &SEQ_DOUBLE_SDFFE;

        let none = Config::default();
        let _ = find_subgraphs(pat, hay, &none);

        let auto_morph = ConfigBuilder::default()
            .dedupe(DedupeMode::AutoMorph)
            .build();
        let _ = find_subgraphs(pat, hay, &auto_morph);
    }
}
