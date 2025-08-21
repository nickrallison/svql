use std::collections::{HashMap, HashSet};

use log::trace;
use prjunnamed_netlist::Design;

pub mod config;
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

pub use config::{Config, DedupeMode};
pub use model::{CellWrapper, ValidCellWrapper};

use crate::dedupe::{signature_mapped_gate_set, signature_with_boundary};
use crate::index::Index;
use crate::model::{get_input_cells, get_output_cells};
use crate::search::heuristics::ChosenCellSelection;
use crate::search::{backtrack, rarest_gate_heuristic};
use crate::state::State;

#[derive(Clone, Debug)]
pub struct AllSubgraphMatches {
    pub matches: Vec<SubgraphMatch>,
}

impl AllSubgraphMatches {
    pub fn len(&self) -> usize {
        self.matches.len()
    }
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, SubgraphMatch> {
        self.matches.iter()
    }
    pub fn first(&self) -> Option<&SubgraphMatch> {
        self.matches.first()
    }
}

impl IntoIterator for AllSubgraphMatches {
    type Item = SubgraphMatch;
    type IntoIter = std::vec::IntoIter<SubgraphMatch>;
    fn into_iter(self) -> Self::IntoIter {
        self.matches.into_iter()
    }
}

impl<'a> IntoIterator for &'a AllSubgraphMatches {
    type Item = &'a SubgraphMatch;
    type IntoIter = std::slice::Iter<'a, SubgraphMatch>;
    fn into_iter(self) -> Self::IntoIter {
        self.matches.iter()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch {
    pub cell_mapping: HashMap<CellWrapper, CellWrapper>,
    pub pat_input_cells: Vec<CellWrapper>,
    pub pat_output_cells: Vec<CellWrapper>,
    pub boundary_src_map: HashMap<(CellWrapper, usize), (CellWrapper, usize)>,

    // this is by the pattern so it is hopefully small enough to not be a problem cloning
    pub input_by_name: HashMap<String, CellWrapper>,
    pub output_by_name: HashMap<String, CellWrapper>,
    pub out_driver_map: HashMap<(CellWrapper, usize), (CellWrapper, usize)>,
}

impl SubgraphMatch {
    pub fn len(&self) -> usize {
        self.cell_mapping.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cell_mapping.is_empty()
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, CellWrapper, CellWrapper> {
        self.cell_mapping.iter()
    }

    pub fn design_source_of_input_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellWrapper, usize)> {
        let p_in = *self.input_by_name.get(name)?;
        self.boundary_src_map.get(&(p_in, bit)).copied()
    }

    pub fn design_driver_of_output_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellWrapper, usize)> {
        let p_out = *self.output_by_name.get(name)?;
        self.out_driver_map.get(&(p_out, bit)).copied()
    }
}

// Main Public API
pub fn find_subgraphs(pattern: &Design, design: &Design, config: &Config) -> AllSubgraphMatches {
    trace!("Starting find_subgraphs with config: {:?}", config);

    let p_index = Index::build(pattern);
    trace!("Built pattern index with {} gates", p_index.gate_count());

    let d_index = Index::build(design);
    trace!("Built design index with {} gates", d_index.gate_count());

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        trace!("Early exit: no gates in pattern or design");
        return AllSubgraphMatches {
            matches: Vec::new(),
        };
    }

    let Some(anchor @ ChosenCellSelection { .. }) = rarest_gate_heuristic(&p_index, &d_index)
    else {
        trace!("No anchor found - early exit");
        return AllSubgraphMatches {
            matches: Vec::new(),
        };
    };
    trace!("Selected anchor: {:?}", anchor);

    let mut results: Vec<SubgraphMatch> = Vec::new();
    let (pat_inputs, pat_outputs) = get_pattern_io_cells(pattern, d_index.design_hash());
    trace!(
        "Pattern IO cells - inputs: {}, outputs: {}",
        pat_inputs.len(),
        pat_outputs.len()
    );

    // Deterministically pick a single pattern anchor (the minimum NodeId).
    let p_a = *anchor
        .pat_anchors
        .iter()
        .min()
        .expect("No pattern anchors found");
    trace!("Using pattern anchor node: {}", p_a);

    for &d_a in &anchor.des_anchors {
        trace!("Trying design anchor node: {}", d_a);

        let empty_state = State::new(p_index.gate_count(), pattern, design);
        if !crate::state::cells_compatible(
            p_a,
            d_a,
            &p_index,
            &d_index,
            &empty_state,
            config.match_length,
        ) {
            trace!("Anchor pair incompatible - skipping");
            continue;
        }
        trace!("Anchor pair compatible");

        let mut st = State::new(p_index.gate_count(), pattern, design);
        st.map(p_a, d_a);
        trace!("Mapped anchor {} -> {}", p_a, d_a);

        let added = search::add_bindings_from_pair(p_a, d_a, &p_index, &d_index, &mut st, config);
        trace!("Added {} bindings from anchor pair", added.len());

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
        trace!("Backtracked from anchor {}", d_a);
    }
    trace!("Found {} raw matches", results.len());

    match config.dedupe {
        DedupeMode::None => {
            let mut seen = HashSet::new();
            let before = results.len();
            results.retain(|m| seen.insert(signature_with_boundary(m)));
            trace!(
                "Deduped from {} to {} matches (None mode)",
                before,
                results.len()
            );
        }
        DedupeMode::AutoMorph => {
            let mut seen = std::collections::HashSet::new();
            let before = results.len();
            results.retain(|m| seen.insert(signature_mapped_gate_set(m)));
            trace!(
                "Deduped from {} to {} matches (AutoMorph mode)",
                before,
                results.len()
            );
        }
    }

    trace!("Returning {} final matches", results.len());
    AllSubgraphMatches { matches: results }
}

// Internal helper for tests and internal wiring only.
pub(crate) fn get_pattern_io_cells(
    pattern: &Design,
    design_hash: u64,
) -> (Vec<CellWrapper>, Vec<CellWrapper>) {
    (
        get_input_cells(pattern, design_hash),
        get_output_cells(pattern, design_hash),
    )
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use crate::{
        config::{Config, ConfigBuilder, DedupeMode},
        index::calculate_design_hash,
    };

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
        let design_hash = calculate_design_hash(design);
        let (ins, outs) = get_pattern_io_cells(design, design_hash);
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
