use std::collections::HashMap;

use prjunnamed_netlist::Design;

mod dedupe;
mod index;
mod model;
mod search;
mod state;

// Simplified “wildcard-IO” mode
mod simple;

pub use model::CellWrapper;
use svql_common::{Config, DedupeMode};

use crate::dedupe::signature_mapped_gate_set;
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
    // Gate mapping (pattern gate -> design gate)
    pub cell_mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>>,

    // Simplified IO view:
    // - pattern input name -> design cell that feeds it (if uniquely determined)
    // - pattern output name -> design cell that drives it (if uniquely determined)
    pub input_binding_by_name: HashMap<&'p str, CellWrapper<'d>>,
    pub output_driver_by_name: HashMap<&'p str, CellWrapper<'d>>,
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

    if config.dedupe == DedupeMode::AutoMorph {
        let mut seen = std::collections::HashSet::new();
        results.retain(|m| seen.insert(signature_mapped_gate_set(m)));
    }

    AllSubgraphMatches { matches: results }
}

/// Simplified public API: wildcard inputs, 1-to-1 gate mapping by kind, no external binding unification.
pub fn find_subgraphs_simple<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> AllSubgraphMatches<'p, 'd> {
    simple::find_subgraphs_simple(pattern, design, config)
}

// Internal helper for tests and internal wiring only.
pub(crate) fn get_pattern_io_cells<'p>(
    pattern: &'p Design,
) -> (Vec<CellWrapper<'p>>, Vec<CellWrapper<'p>>) {
    (get_input_cells(pattern), get_output_cells(pattern))
}
