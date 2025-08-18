use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Design;

pub mod cell_kind;
use cell_kind::{InputCell, OutputCell, get_input_cells, get_output_cells};

use crate::cell_kind::CellWrapper;

mod anchor;
mod compat;
mod index;
mod ports;
mod search;
mod state;
mod strategy;
pub(crate) mod util;

#[derive(Clone, Debug)]
pub struct AllSubgraphMatches<'p, 'd> {
    pub matches: Vec<SubgraphMatch<'p, 'd>>,
    pub _p_index: index::Index<'p>,
    pub _d_index: index::Index<'d>,
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
}

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    pub cell_mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    pub pat_input_cells: Vec<InputCell<'p>>,
    pub pat_output_cells: Vec<OutputCell<'p>>,
    pub boundary_src_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)>,

    // lookup indices
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

fn match_signature<'p, 'd>(m: &SubgraphMatch<'p, 'd>) -> Vec<(u8, usize, usize, usize, usize)> {
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

// Public API
pub fn find_subgraphs<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
) -> AllSubgraphMatches<'p, 'd> {
    let p_index = index::Index::build(pattern);
    let d_index = index::Index::build(design);

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        return AllSubgraphMatches {
            matches: Vec::new(),
            _p_index: p_index,
            _d_index: d_index,
        };
    }

    let Some((_anchor_kind, p_anchors, d_anchors)) = anchor::choose_anchors(&p_index, &d_index)
    else {
        return AllSubgraphMatches {
            matches: Vec::new(),
            _p_index: p_index,
            _d_index: d_index,
        };
    };

    let mut results: Vec<SubgraphMatch<'p, 'd>> = Vec::new();
    let (pat_inputs, pat_outputs) = get_pattern_io_cells(pattern);

    // Canonicalize pattern anchor to avoid multiplicity
    let p_anchor = *p_anchors.iter().min().unwrap();
    let p_anchors = vec![p_anchor];

    for &p_a in &p_anchors {
        for &d_a in &d_anchors {
            if p_index.kind(p_a) != d_index.kind(d_a) {
                continue;
            }
            let empty_state = state::State::<'p, 'd>::new(p_index.gate_count());
            if !compat::cells_compatible(p_a, d_a, &p_index, &d_index, &empty_state) {
                continue;
            }

            let mut st = state::State::new(p_index.gate_count());
            st.map(p_a, d_a);

            // Add IO boundaries implied by anchor mapping
            let added = search::add_io_boundaries_from_pair(p_a, d_a, &p_index, &d_index, &mut st);

            search::backtrack(
                &p_index,
                &d_index,
                &mut st,
                &mut results,
                &pat_inputs,
                &pat_outputs,
            );

            // Backtrack anchor boundaries
            search::remove_boundaries(added, &mut st);
            st.unmap(p_a, d_a);
        }
    }

    // Dedupe by combined signature
    let mut seen: HashSet<Vec<(u8, usize, usize, usize, usize)>> = HashSet::new();
    results.retain(|m| seen.insert(match_signature(m)));

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
    use super::*;

    lazy_static::lazy_static! {
        static ref ASYNC_MUX: Design = crate::util::load_design_from("examples/patterns/security/access_control/locked_reg/json/async_mux.json").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/seq_double_sdffe.v").unwrap();
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/sdffe.v").unwrap();
        static ref COMB_D_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/comb_d_double_sdffe.v").unwrap();
        static ref PAR_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/par_double_sdffe.v").unwrap();

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
        let matches = find_subgraphs(design, design);
        assert!(
            !matches.is_empty(),
            "Self-match sdffe should yield at least one mapping"
        );
        for m in matches.iter() {
            assert!(!m.is_empty());
        }
    }

    #[test]
    fn smoke_seq_double_sdffe_has_at_least_one() {
        let design = &SEQ_DOUBLE_SDFFE;
        let matches = find_subgraphs(design, design);
        assert!(
            !matches.is_empty(),
            "Self-match seq_double_sdffe should yield mappings"
        );
    }

    #[test]
    fn exact_two_matches_comb_d_double_self() {
        let design = &COMB_D_DOUBLE_SDFFE;
        let matches = find_subgraphs(design, design);
        assert_eq!(
            matches.len(),
            2,
            "canonical anchor + dedupe should yield 2 mappings"
        );
    }

    #[test]
    fn exact_two_matches_sdffe_in_seq_double() {
        let pat = &SDFFE;
        let hay = &SEQ_DOUBLE_SDFFE;
        let matches = find_subgraphs(pat, hay);
        assert_eq!(
            matches.len(),
            2,
            "pattern IO should bind to gate, yielding 2 matches"
        );
    }

    #[test]
    fn dedupe_eliminates_anchor_duplicates_par_double_self() {
        let design = &PAR_DOUBLE_SDFFE;
        let matches = find_subgraphs(design, design);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn o1_lookup_by_port_name_and_bit_sdffe_in_seq_double() {
        let pat = &SDFFE;
        let hay = &SEQ_DOUBLE_SDFFE;
        let all = find_subgraphs(pat, hay);
        assert_eq!(
            all.len(),
            2,
            "expected two sdffe matches in seq_double_sdffe"
        );

        // Every match should resolve both d (input) and q (output) via O(1) helpers
        for m in all.iter() {
            assert!(
                m.design_source_of_input_bit("d", 0).is_some(),
                "input d should have a bound design source"
            );
            assert!(
                m.design_driver_of_output_bit("q", 0).is_some(),
                "output q should have a resolved design driver"
            );
        }

        // There should exist a pair of matches where q of one drives d of the other.
        let mut found = false;
        let ms: Vec<_> = all.iter().collect();
        for m1 in &ms {
            if let Some((dq_cell, dq_bit)) = m1.design_driver_of_output_bit("q", 0) {
                let dq_net = dq_cell.output()[dq_bit];
                for m2 in &ms {
                    if let Some((sd_cell, sd_bit)) = m2.design_source_of_input_bit("d", 0) {
                        let sd_net = sd_cell.output()[sd_bit];
                        if dq_net == sd_net {
                            found = true;
                            break;
                        }
                    }
                }
            }
            if found {
                break;
            }
        }
        assert!(
            found,
            "expected to find at least one connection: q of one match drives d of another"
        );
    }
}
