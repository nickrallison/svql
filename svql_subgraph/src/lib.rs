// mod dedupe;
mod index;
mod model;
mod search;
// mod state;

pub use model::CellWrapper;
use svql_common::Config;

use crate::index::{Index, NodeId};
use crate::search::rarest_gate_heuristic;

use core::panic;
use std::cell;
use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Design;

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    /// Gate mapping (pattern gate -> design gate)
    pub cell_mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    /// pattern input name -> design cell that feeds it
    pub input_binding_by_name: HashMap<&'p str, Vec<(CellWrapper<'d>, usize)>>,
    /// pattern output name -> design cell that drives it
    pub output_driver_by_name: HashMap<&'p str, Vec<(CellWrapper<'d>, usize)>>,
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

pub fn find_subgraphs<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> Vec<SubgraphMatch<'p, 'd>> {
    let p_index = Index::build(pattern);
    let d_index = Index::build(design);

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        return Vec::new();
    }

    let mut results: Vec<SubgraphMatch<'p, 'd>> = Vec::new();

    // in topological_order, only gates & inputs
    let p_mapping_queue: Vec<&CellWrapper<'p>> = p_index
        .get_cells_topo()
        .into_iter()
        .filter(|c| !matches!(c.kind, model::CellKind::Output))
        .collect();

    panic!("{:#?}", p_mapping_queue);

    return results;

    // for &d_a in &des_anchors {
    //     let empty_state = State::<'p, 'd>::new(p_index.gate_count());
    //     if !cells_compatible(
    //         p_a,
    //         d_a,
    //         &p_index,
    //         &d_index,
    //         &empty_state,
    //         config.match_length,
    //     ) {
    //         continue;
    //     }

    //     let mut st = State::new(p_index.gate_count());
    //     st.map(p_a, d_a);
    //     let added = search::add_bindings_from_pair(p_a, d_a, &p_index, &d_index, &mut st, config);

    //     backtrack(
    //         &p_index,
    //         &d_index,
    //         &mut st,
    //         &mut results,
    //         &pat_inputs,
    //         &pat_outputs,
    //         config,
    //     );

    //     search::remove_bindings(added, &mut st);
    //     st.unmap(p_a, d_a);
    // }

    // if config.dedupe == DedupeMode::AutoMorph {
    //     let mut seen = std::collections::HashSet::new();
    //     results.retain(|m| seen.insert(signature_mapped_gate_set(m)));
    // }

    // AllSubgraphMatches { matches: results }
    todo!()
}

fn recursive_search<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    config: &Config,
    mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    mapping_queue: Vec<CellWrapper<'p>>,
) -> Vec<SubgraphMatch<'p, 'd>> {
    todo!()
}
