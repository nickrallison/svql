mod index;
mod mapping;
mod model;

use index::Index;
use mapping::CellMapping;
use prjunnamed_netlist::Design;
use std::collections::VecDeque;
use svql_common::Config;

use crate::model::CellWrapper;

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    mapping: CellMapping<'p, 'd>,
}

impl<'p, 'd> SubgraphMatch<'p, 'd> {
    pub fn len(&self) -> usize {
        self.mapping.len()
    }
    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    // pub fn pattern_mapping(&self) -> &HashMap<&'p CellWrapper<'p>, &'d CellWrapper<'d>> {
    //     self.mapping.pattern_mapping()
    // }
    // pub fn design_mapping(&self) -> &HashMap<&'d CellWrapper<'d>, &'p CellWrapper<'p>> {
    //     self.mapping.design_mapping()
    // }
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
        .filter(|c| !matches!(c.kind, model::CellKind::Output))
        .map(|c| c.clone())
        .collect();

    let initial_cell_mapping: CellMapping<'p, 'd> = CellMapping::new();

    find_subgraphs_recursive(
        &p_index,
        &d_index,
        config,
        initial_cell_mapping,
        p_mapping_queue,
    )
}

fn find_subgraphs_recursive<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    config: &Config,
    cell_mapping: CellMapping<'p, 'd>,
    mut p_mapping_queue: VecDeque<CellWrapper<'p>>,
) -> Vec<SubgraphMatch<'p, 'd>> {
    // 1. Pop the first element of mapping queue
    let Some(current) = p_mapping_queue.pop_front() else {
        return vec![SubgraphMatch {
            mapping: cell_mapping,
        }];
    };

    // 2. Find All Possible Mappings
    //     - Must not be already mapped
    //     - Must be compatible
    //     - Design must share the same connectivity as the pattern
    //         - See pins field of CellWrapper

    let all_possible_matches = d_index.get_by_kind(current.kind).into_iter();

    let not_already_mapped = all_possible_matches
        .filter(|&d_cell| !cell_mapping.design_mapping().contains_key(&d_cell.cref()));

    let compatible = not_already_mapped
        .filter(|&d_cell| cells_compatible(&current, d_cell, p_index, d_index, config));

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

    let results: Vec<SubgraphMatch<'p, 'd>> = {
        let mut results: Vec<SubgraphMatch<'p, 'd>> = new_cell_mappings
            .into_iter()
            .flat_map(|new_cell_mapping| {
                find_subgraphs_recursive(
                    p_index,
                    d_index,
                    config,
                    new_cell_mapping,
                    p_mapping_queue.clone(),
                )
            })
            .collect();
        let mut seen = std::collections::HashSet::new();
        results.retain(|m| seen.insert(m.mapping.sig()));

        results
    };

    results
}

fn cells_compatible<'p, 'd>(
    p_cell: &CellWrapper<'p>,
    d_cell: &CellWrapper<'d>,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    config: &Config,
) -> bool {
    // Check if the cells share the same kind
    if p_cell.kind != d_cell.kind {
        return false;
    }

    // later check config for size compatibility

    true
}

fn cells_share_connectivity<'p, 'd>(
    p_cell: &CellWrapper<'p>,
    d_cell: &CellWrapper<'d>,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    _config: &Config,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    // Downstream check:
    // For each already-mapped fanout (sink) of the pattern cell, ensure the design cell
    // also drives the mapped sink at the same pin index.
    let d_fanouts = d_index.get_fanouts(d_cell.cref());
    p_index
        .get_fanouts(p_cell.cref())
        .iter()
        .filter_map(|(p_sink_cell, pin_idx)| {
            mapping
                .get_design_cell(*p_sink_cell)
                .map(|d_sink_cell| (d_sink_cell, *pin_idx))
        })
        .all(|(d_sink_cell, pin_idx)| {
            d_fanouts
                .iter()
                .any(|(s, i)| *s == d_sink_cell && *i == pin_idx)
        })
}
