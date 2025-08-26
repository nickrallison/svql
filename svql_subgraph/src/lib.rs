mod index;
mod mapping;
mod model;

use index::Index;
use log::trace;
use mapping::CellMapping;
use prjunnamed_netlist::Design;
use std::collections::VecDeque;
use svql_common::Config;

use crate::model::{CellKind, CellWrapper};

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
    trace!(
        "find_subgraphs: start. pattern cells={} design cells={}",
        pattern.iter_cells().count(),
        design.iter_cells().count()
    );

    let p_index = Index::build(pattern);
    let d_index = Index::build(design);

    trace!(
        "find_subgraphs: gate counts: pattern={} design={}",
        p_index.gate_count(),
        d_index.gate_count()
    );

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        trace!("find_subgraphs: early return (empty gate count)");
        return Vec::new();
    }

    // in topological_order, only gates & inputs
    let p_mapping_queue: VecDeque<CellWrapper<'p>> = p_index
        .get_cells_topo()
        .into_iter()
        .filter(|c| !matches!(c.kind, model::CellKind::Output))
        .map(|c| c.clone())
        .collect();

    trace!(
        "find_subgraphs: initial pattern mapping queue size={}",
        p_mapping_queue.len()
    );

    let initial_cell_mapping: CellMapping<'p, 'd> = CellMapping::new();

    let results = find_subgraphs_recursive(
        &p_index,
        &d_index,
        config,
        initial_cell_mapping,
        p_mapping_queue,
    );

    trace!("find_subgraphs: results={}", results.len());
    results
}

// fn find_subgraphs_recursive<'p, 'd>(
//     p_index: &Index<'p>,
//     d_index: &Index<'d>,
//     config: &Config,
//     cell_mapping: CellMapping<'p, 'd>,
//     mut p_mapping_queue: VecDeque<CellWrapper<'p>>,
//     depth: usize,
// ) -> Vec<SubgraphMatch<'p, 'd>> {
//     let Some(current) = p_mapping_queue.pop_front() else {
//         trace!(
//             "find_subgraphs_recursive[depth={}]: base case reached. mapping size={}",
//             depth,
//             cell_mapping.len()
//         );
//         return vec![SubgraphMatch {
//             mapping: cell_mapping,
//         }];
//     };

//     trace!(
//         "find_subgraphs_recursive[depth={}]: current pattern cell {}",
//         depth,
//         current.summary()
//     );

//     // Enumerate candidates with detailed filtering diagnostics.
//     let mut new_cell_mappings: Vec<CellMapping<'p, 'd>> = Vec::new();
//     let mut total_candidates = 0usize;
//     let mut already_mapped = 0usize;
//     let mut incompatible = 0usize;
//     let mut connectivity_fail = 0usize;

//     // IMPORTANT: If the current pattern node is an Input, allow it to map to ANY design node.
//     // This permits pattern inputs to bind to DFF outputs, gate outputs, etc., not only top-level inputs.
//     // For other kinds, keep the kind-based filtering for performance.
//     let d_candidates: Vec<&CellWrapper<'d>> = if matches!(current.kind, CellKind::Input) {
//         d_index.get_cells_topo().iter().collect()
//     } else {
//         d_index.get_by_kind(current.kind).iter().collect()
//     };

//     for d_cell in d_candidates {
//         total_candidates += 1;

//         if cell_mapping.design_mapping().contains_key(&d_cell.cref()) {
//             already_mapped += 1;
//             trace!(
//                 "find_subgraphs_recursive[depth={}]: skip D {} (already mapped)",
//                 depth,
//                 d_cell.summary()
//             );
//             continue;
//         }

//         if !cells_compatible(&current, d_cell, p_index, d_index, config) {
//             incompatible += 1;
//             trace!(
//                 "find_subgraphs_recursive[depth={}]: skip D {} (incompatible with P {})",
//                 depth,
//                 d_cell.summary(),
//                 current.summary()
//             );
//             continue;
//         }

//         let shares =
//             cells_share_connectivity(&current, d_cell, p_index, d_index, config, &cell_mapping);

//         if !shares {
//             connectivity_fail += 1;
//             trace!(
//                 "find_subgraphs_recursive[depth={}]: skip D {} (connectivity mismatch with P {})",
//                 depth,
//                 d_cell.summary(),
//                 current.summary()
//             );
//             continue;
//         }

//         let mut new_cm = cell_mapping.clone();
//         new_cm.insert(current.cref(), d_cell.cref());
//         trace!(
//             "find_subgraphs_recursive[depth={}]: ACCEPT mapping P {} -> D {} (mapping size now {})",
//             depth,
//             current.summary(),
//             d_cell.summary(),
//             new_cm.len()
//         );
//         new_cell_mappings.push(new_cm);
//     }

//     trace!(
//         "find_subgraphs_recursive[depth={}]: candidate stats: total={} already_mapped={} incompatible={} connectivity_fail={} accepted={}",
//         depth,
//         total_candidates,
//         already_mapped,
//         incompatible,
//         connectivity_fail,
//         new_cell_mappings.len()
//     );

//     let mut results: Vec<SubgraphMatch<'p, 'd>> = new_cell_mappings
//         .into_iter()
//         .flat_map(|new_cell_mapping| {
//             find_subgraphs_recursive(
//                 p_index,
//                 d_index,
//                 config,
//                 new_cell_mapping,
//                 p_mapping_queue.clone(),
//                 depth + 1,
//             )
//         })
//         .collect();

//     let before_dedup = results.len();
//     let mut seen = std::collections::HashSet::new();
//     results.retain(|m| seen.insert(m.mapping.sig()));
//     let after_dedup = results.len();

//     trace!(
//         "find_subgraphs_recursive[depth={}]: results before_dedup={} after_dedup={}",
//         depth, before_dedup, after_dedup
//     );

//     results
// }

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
    _p_index: &Index<'p>,
    _d_index: &Index<'d>,
    _config: &Config,
) -> bool {
    // Check if the cells share the same kind
    let kind_matches = match (p_cell.kind, d_cell.kind) {
        (CellKind::Input, _d_cell_kind) => return true,
        _ => p_cell.kind == d_cell.kind,
    };

    if !kind_matches {
        trace!(
            "cells_compatible: kind mismatch P {} vs D {}",
            p_cell.summary(),
            d_cell.summary()
        );
        return false;
    }

    // TODO: later check config for size compatibility
    trace!(
        "cells_compatible: OK P {} <-> D {}",
        p_cell.summary(),
        d_cell.summary()
    );

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
    // For each already-mapped fanout (sink) of the pattern cell, ensure the design cell
    // also drives the mapped sink at the same pin index.

    let d_fanouts = d_index.get_fanouts(d_cell.cref());

    let p_fanouts = p_index.get_fanouts(p_cell.cref());
    trace!(
        "cells_share_connectivity: P {} fanouts={} | D {} fanouts={}",
        p_cell.summary(),
        p_fanouts.len(),
        d_cell.summary(),
        d_fanouts.len()
    );

    let result = p_fanouts
        .iter()
        .filter_map(|(p_sink_cell, pin_idx)| {
            mapping
                .get_design_cell(*p_sink_cell)
                .map(|d_sink_cell| (d_sink_cell, *pin_idx))
        })
        .all(|(d_sink_cell, pin_idx)| {
            let ok = d_fanouts
                .iter()
                .any(|(s, i)| *s == d_sink_cell && *i == pin_idx);
            trace!(
                "cells_share_connectivity: check mapped sink D#{} @pin={} -> {}",
                d_sink_cell.debug_index(),
                pin_idx,
                ok
            );
            ok
        });

    trace!(
        "cells_share_connectivity: P {} vs D {} -> {}",
        p_cell.summary(),
        d_cell.summary(),
        result
    );
    result
}
