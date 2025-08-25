// svql_subgraph/src/lib.rs
mod index;
mod mapping;
mod model;

use index::Index;
use mapping::CellMapping;
use prjunnamed_netlist::Design;
use std::collections::{HashMap, VecDeque};
use svql_common::Config;

use crate::model::{CellKind, CellWrapper};

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    /// Cell Mapping from pattern cells to design cells
    mapping: HashMap<prjunnamed_netlist::CellRef<'p>, prjunnamed_netlist::CellRef<'d>>,
    // /// pattern input name -> pattern cell that feeds it
    // input_binding_by_name: HashMap<String, Vec<(prjunnamed_netlist::CellRef<'p>, usize)>>,
    // /// pattern output name -> pattern cell that drives it
    // output_driver_by_name: HashMap<String, Vec<(prjunnamed_netlist::CellRef<'p>, usize)>>,
}

impl<'p, 'd> SubgraphMatch<'p, 'd> {
    pub fn len(&self) -> usize {
        self.mapping.len()
    }
    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub fn mapping(
        &self,
    ) -> &HashMap<prjunnamed_netlist::CellRef<'p>, prjunnamed_netlist::CellRef<'d>> {
        &self.mapping
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

    // in topological_order, only gates & inputs
    let p_mapping_queue: VecDeque<CellWrapper<'p>> = p_index
        .get_cells_topo()
        .into_iter()
        .filter(|c| !matches!(c.kind, CellKind::Output))
        .map(|c| c.clone())
        .collect();

    let initial_cell_mapping: CellMapping<'p, 'd> = CellMapping::new();

    find_subgraphs_recursive(
        pattern,
        design,
        &p_index,
        &d_index,
        config,
        initial_cell_mapping,
        p_mapping_queue,
    )
}

fn find_subgraphs_recursive<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    config: &Config,
    cell_mapping: CellMapping<'p, 'd>,
    mut p_mapping_queue: VecDeque<CellWrapper<'p>>,
) -> Vec<SubgraphMatch<'p, 'd>> {
    // 1. Pop the first element of mapping queue
    let current = match p_mapping_queue.pop_front() {
        Some(cell) => cell,
        None => {
            // Base Case: If there are no more cells to be mapped, construct a match from the given cell_mapping
            // Convert the CellMapping to a SubgraphMatch
            let mapping = cell_mapping.into_mapping();
            let match_result = SubgraphMatch { mapping };
            return vec![match_result];
        }
    };

    // 2. Find All Possible Mappings
    //     - Must not be already mapped
    //     - Must be compatible
    //     - Design must share the same connectivity as the pattern
    //         - See pins field of CellWrapper

    let all_possible_matches = d_index.get_by_kind(current.kind).into_iter();

    let not_already_mapped = all_possible_matches
        .filter(|&d_cell| !cell_mapping.design_mapping().contains_key(&d_cell.cref()));

    let compatible =
        not_already_mapped.filter(|&d_cell| cells_compatible(&current, d_cell, config));

    let shares_connectivity = compatible.filter(|&d_cell| {
        cells_share_connectivity(&current, d_cell, p_index, d_index, config, &cell_mapping)
    });

    // 3. For each valid cell, create a new mapping and try to call self recursively and put together all the results
    //     - This looks like a lot, but the `cells_share_connectivity` function should heavily cut down the number of recursions
    let new_cell_mappings: Vec<CellMapping<'p, 'd>> = shares_connectivity
        .map(|d_cell| {
            let mut new_cell_mapping = cell_mapping.clone();
            new_cell_mapping.insert(current.cref(), d_cell.cref());
            new_cell_mapping
        })
        .collect();

    let results: Vec<SubgraphMatch<'p, 'd>> = new_cell_mappings
        .into_iter()
        .flat_map(|new_cell_mapping| {
            find_subgraphs_recursive(
                pattern,
                design,
                p_index,
                d_index,
                config,
                new_cell_mapping,
                p_mapping_queue.clone(),
            )
        })
        .collect();

    // Deduplicate results based on the mapping signature
    let mut seen = std::collections::HashSet::new();
    let mut deduped_results = Vec::new();
    for result in results {
        // Create a signature based on the cell indices in the design
        let mut sig: Vec<usize> = result.mapping.values().map(|c| c.debug_index()).collect();
        sig.sort_unstable();
        sig.dedup();

        if seen.insert(sig) {
            deduped_results.push(result);
        }
    }

    deduped_results
}

fn cells_compatible<'p, 'd>(
    p_cell: &CellWrapper<'p>,
    d_cell: &CellWrapper<'d>,
    _config: &Config,
) -> bool {
    // Check if the cells share the same kind
    if p_cell.kind != d_cell.kind {
        return false;
    }

    // TODO: Check other compatibility criteria based on config
    // For example, check pin counts if match_length is true

    true
}

fn cells_share_connectivity<'p, 'd>(
    p_cell: &CellWrapper<'p>,
    d_cell: &CellWrapper<'d>,
    _p_index: &Index<'p>,
    _d_index: &Index<'d>,
    _config: &Config,
    _mapping: &CellMapping<'p, 'd>,
) -> bool {
    todo!()
}
