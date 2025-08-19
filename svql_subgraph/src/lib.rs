use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Design;

pub mod cell_kind;
use cell_kind::{get_input_cells, get_output_cells};

mod anchor;
mod compat;
pub mod config;
mod index;
mod ports;
mod search;
mod state;
mod strategy;
pub(crate) mod util;

// Public re-exports for a minimal, stable external API.
pub use cell_kind::{CellWrapper, InputCell, OutputCell};
pub use config::{Config, DedupeMode};

// Also re-export under their original short names for ergonomic use in dependents.
// pub use config::{Config, DedupeMode};

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
    pub pat_input_cells: Vec<InputCell<'p>>,
    pub pat_output_cells: Vec<OutputCell<'p>>,
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

pub fn find_subgraphs<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> AllSubgraphMatches<'p, 'd> {
    let p_index = index::Index::build(pattern);
    let d_index = index::Index::build(design);

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        return AllSubgraphMatches {
            matches: Vec::new(),
        };
    }

    let Some((_anchor_kind, p_anchors, d_anchors)) = anchor::choose_anchors(&p_index, &d_index)
    else {
        return AllSubgraphMatches {
            matches: Vec::new(),
        };
    };

    let mut results: Vec<SubgraphMatch<'p, 'd>> = Vec::new();
    let (pat_inputs, pat_outputs) = get_pattern_io_cells(pattern);

    let p_anchor = *p_anchors.iter().min().unwrap();
    let p_anchors = vec![p_anchor];
    let p_a = *p_anchors.first().expect("No pattern anchors found");

    for &d_a in &d_anchors {
        if p_index.kind(p_a) != d_index.kind(d_a) {
            continue;
        }
        let empty_state = state::State::<'p, 'd>::new(p_index.gate_count());
        if !compat::cells_compatible(
            p_a,
            d_a,
            &p_index,
            &d_index,
            &empty_state,
            config.match_length,
        ) {
            continue;
        }

        let mut st = state::State::new(p_index.gate_count());
        st.map(p_a, d_a);
        let added =
            search::add_io_boundaries_from_pair(p_a, d_a, &p_index, &d_index, &mut st, config);

        search::backtrack(
            &p_index,
            &d_index,
            &mut st,
            &mut results,
            &pat_inputs,
            &pat_outputs,
            &config,
        );

        search::remove_boundaries(added, &mut st);
        st.unmap(p_a, d_a);
    }

    // Dedupe according to config
    match config.dedupe {
        DedupeMode::Full => {
            let mut seen: HashSet<Vec<(u8, usize, usize, usize, usize)>> = HashSet::new();
            results.retain(|m| seen.insert(signature_full(m)));
        }
        DedupeMode::GatesOnly => {
            // Collapse matches that select the same set of design gates, regardless of
            // which pattern gates they map to (ignores boundary bindings and automorphisms).
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(signature_gates_only(m)));
        }
    }

    AllSubgraphMatches { matches: results }
}

pub fn get_pattern_io_cells<'p>(pattern: &'p Design) -> (Vec<InputCell<'p>>, Vec<OutputCell<'p>>) {
    (get_input_cells(pattern), get_output_cells(pattern))
}

fn signature_full<'p, 'd>(m: &SubgraphMatch<'p, 'd>) -> Vec<(u8, usize, usize, usize, usize)> {
    let mut sig: Vec<(u8, usize, usize, usize, usize)> = Vec::new();

    for (p, d) in m.cell_mapping.iter() {
        sig.push((0, p.debug_index(), 0, d.debug_index(), 0));
    }
    for ((p_cell, p_bit), (d_cell, d_bit)) in m.boundary_src_map.iter() {
        sig.push((
            1,
            p_cell.debug_index(),
            *p_bit,
            d_cell.debug_index(),
            *d_bit,
        ));
    }
    sig.sort_unstable();
    sig
}

fn signature_gates_only<'p, 'd>(m: &SubgraphMatch<'p, 'd>) -> Vec<usize> {
    // Only the set of design gate IDs matter; ignore which pattern gates map to them
    // and ignore any boundary bindings. This collapses automorphisms/permutations.
    let mut sig: Vec<usize> = m.cell_mapping.values().map(|d| d.debug_index()).collect();
    sig.sort_unstable();
    sig.dedup();
    sig
}

#[cfg(test)]
mod tests {
    use crate::{config::ConfigBuilder, util::load_design_from};

    use super::*;

    lazy_static::lazy_static! {
        static ref ASYNC_MUX: Design = crate::util::load_design_from("examples/patterns/security/access_control/locked_reg/json/async_mux.json").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v").unwrap();
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
        static ref COMB_D_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v").unwrap();
        static ref PAR_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/fixtures/basic/ff/verilog/par_double_sdffe.v").unwrap();
    }

    #[test]
    fn smoke_io_cells() {
        let design = &ASYNC_MUX;
        let (ins, outs) = get_pattern_io_cells(&design);
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

        let full = Config::default();
        let _ = find_subgraphs(pat, hay, &full);

        let gates_only = ConfigBuilder::default()
            .dedupe(DedupeMode::GatesOnly)
            .build();
        let _ = find_subgraphs(pat, hay, &gates_only);
    }
}
