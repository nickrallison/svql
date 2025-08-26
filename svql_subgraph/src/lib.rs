mod cell;
mod index;
mod mapping;

use index::Index;
use log::{info, trace};
use mapping::CellMapping;
use prjunnamed_netlist::Design;
use std::collections::{HashMap, VecDeque};
use svql_common::{Config, DedupeMode};

use crate::cell::{CellKind, Source};

pub use crate::cell::CellWrapper;

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    // Mapping of pattern cells to design cells (and reverse)
    mapping: CellMapping<'p, 'd>,

    // Boundary IO lookup tables
    pub input_by_name: HashMap<&'p str, CellWrapper<'p>>,
    pub output_by_name: HashMap<&'p str, CellWrapper<'p>>,
}

impl<'p, 'd> SubgraphMatch<'p, 'd> {
    pub fn len(&self) -> usize {
        self.mapping.len()
    }
    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    // Return the design cell that corresponds to the named pattern input bit.
    // For inputs, the design "source" is the cell that the pattern input was mapped to.
    pub fn design_source_of_input_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        let p_input = self.input_by_name.get(name)?;
        let d_src = self.mapping.get_design_cell(p_input.cref())?;
        Some((CellWrapper::new(d_src), bit))
    }

    // Return the design cell that drives the named pattern output bit.
    // We look at the pattern output's input pin driver, then translate that driver
    // through the mapping.
    pub fn design_driver_of_output_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        let p_out = self.output_by_name.get(name)?;
        // Pattern Output should have at least one input pin; pick the requested bit if present.
        let p_src = p_out.pins().get(bit)?;
        match p_src {
            Source::Gate(p_cell, p_bit) | Source::Io(p_cell, p_bit) => {
                let d_cell = self.mapping.get_design_cell(*p_cell)?;
                Some((CellWrapper::new(d_cell), *p_bit))
            }
            Source::Const(_) => None,
        }
    }
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

    // Build boundary IO maps for the pattern (by name).
    let input_by_name: HashMap<&'p str, CellWrapper<'p>> = p_index
        .get_cells_topo()
        .iter()
        .filter(|c| matches!(c.kind(), CellKind::Input))
        .filter_map(|c| c.input_name().map(|n| (n, c.clone())))
        .collect();

    let output_by_name: HashMap<&'p str, CellWrapper<'p>> = p_index
        .get_cells_topo()
        .iter()
        .filter(|c| matches!(c.kind(), CellKind::Output))
        .filter_map(|c| c.output_name().map(|n| (n, c.clone())))
        .collect();

    // in topological_order, only gates & inputs
    let p_mapping_queue: VecDeque<CellWrapper<'p>> = p_index
        .get_cells_topo()
        .into_iter()
        .filter(|c| !matches!(c.kind(), CellKind::Output))
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
        &input_by_name,
        &output_by_name,
        0, // depth
    );

    info!(
        "find_subgraphs: results={} unique_sigs={:?}",
        results.len(),
        results.iter().map(|m| m.mapping.sig()).collect::<Vec<_>>()
    );

    results
}

fn find_subgraphs_recursive<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    config: &Config,
    cell_mapping: CellMapping<'p, 'd>,
    mut p_mapping_queue: VecDeque<CellWrapper<'p>>,
    input_by_name: &HashMap<&'p str, CellWrapper<'p>>,
    output_by_name: &HashMap<&'p str, CellWrapper<'p>>,
    depth: usize,
) -> Vec<SubgraphMatch<'p, 'd>> {
    // 1. Pop the first element of mapping queue
    let Some(current) = p_mapping_queue.pop_front() else {
        trace!(
            "find_subgraphs_recursive[depth={}]: base case reached. mapping size={}",
            depth,
            cell_mapping.len()
        );
        return vec![SubgraphMatch {
            mapping: cell_mapping,
            input_by_name: input_by_name.clone(),
            output_by_name: output_by_name.clone(),
        }];
    };

    trace!(
        "find_subgraphs_recursive[depth={}]: current={} | remaining_queue={} | mapping_size={}",
        depth,
        current.summary(),
        p_mapping_queue.len(),
        cell_mapping.len()
    );

    let mut new_cell_mappings: Vec<CellMapping<'p, 'd>> = Vec::new();

    let mut total_candidates = 0usize;
    let mut already_mapped = 0usize;
    let mut incompatible = 0usize;
    let mut connectivity_fail = 0usize;

    // IMPORTANT: If the current pattern node is an Input, allow it to map to ANY design node.
    let d_candidates: Vec<&CellWrapper<'d>> = if matches!(current.kind(), CellKind::Input) {
        trace!(
            "find_subgraphs_recursive[depth={}]: pattern node is Input; scanning ALL design nodes",
            depth
        );
        d_index.get_cells_topo().iter().collect()
    } else {
        let cands = d_index.get_by_kind(current.kind());
        trace!(
            "find_subgraphs_recursive[depth={}]: filtering by kind {:?}; candidates={}",
            depth,
            current.kind(),
            cands.len()
        );
        cands.iter().collect()
    };

    for d_cell in d_candidates {
        total_candidates += 1;

        if cell_mapping.design_mapping().contains_key(&d_cell.cref()) {
            already_mapped += 1;
            trace!(
                "find_subgraphs_recursive[depth={}]: skip D {} (already mapped)",
                depth,
                d_cell.summary()
            );
            continue;
        }

        if !cells_compatible(&current, d_cell, p_index, d_index, config) {
            incompatible += 1;
            trace!(
                "find_subgraphs_recursive[depth={}]: skip D {} (incompatible with P {})",
                depth,
                d_cell.summary(),
                current.summary()
            );
            continue;
        }

        let shares =
            cells_share_connectivity(&current, d_cell, p_index, d_index, config, &cell_mapping);

        if !shares {
            connectivity_fail += 1;
            trace!(
                "find_subgraphs_recursive[depth={}]: skip D {} (connectivity mismatch with P {})",
                depth,
                d_cell.summary(),
                current.summary()
            );
            continue;
        }

        let mut new_cm = cell_mapping.clone();
        new_cm.insert(current.cref(), d_cell.cref());
        trace!(
            "find_subgraphs_recursive[depth={}]: ACCEPT mapping P {} -> D {} (mapping size now {})",
            depth,
            current.summary(),
            d_cell.summary(),
            new_cm.len()
        );
        new_cell_mappings.push(new_cm);
    }

    trace!(
        "find_subgraphs_recursive[depth={}]: candidate stats: total={} already_mapped={} incompatible={} connectivity_fail={} accepted={}",
        depth,
        total_candidates,
        already_mapped,
        incompatible,
        connectivity_fail,
        new_cell_mappings.len()
    );

    // 3. Recurse on each valid cell and put together all the results, deduplicate at this level.
    let mut results: Vec<SubgraphMatch<'p, 'd>> = new_cell_mappings
        .into_iter()
        .flat_map(|new_cell_mapping| {
            find_subgraphs_recursive(
                p_index,
                d_index,
                config,
                new_cell_mapping,
                p_mapping_queue.clone(),
                input_by_name,
                output_by_name,
                depth + 1,
            )
        })
        .collect();

    if matches!(config.dedupe, DedupeMode::AutoMorph) {
        let before_dedup = results.len();
        let mut seen = std::collections::HashSet::new();
        results.retain(|m| seen.insert(m.mapping.sig()));
        let after_dedup = results.len();

        trace!(
            "find_subgraphs_recursive[depth={}]: results before_dedup={} after_dedup={}",
            depth, before_dedup, after_dedup
        );
    }

    results
}

fn kinds_compatible(p_kind: CellKind, d_kind: CellKind) -> bool {
    // Inputs in the pattern are allowed to map to any design node.
    if matches!(p_kind, CellKind::Input) {
        return true;
    }
    p_kind == d_kind
}

fn cells_compatible<'p, 'd>(
    p_cell: &CellWrapper<'p>,
    d_cell: &CellWrapper<'d>,
    _p_index: &Index<'p>,
    _d_index: &Index<'d>,
    _config: &Config,
) -> bool {
    let compatible = kinds_compatible(p_cell.kind(), d_cell.kind());
    if !compatible {
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

fn source_matches_const<'d>(pin_idx: usize, p_src: &Source<'_>, d_src: &Source<'d>) -> bool {
    let ok = matches!(d_src, Source::Const(dt) if matches!(p_src, Source::Const(pt) if dt == pt));
    if !ok {
        trace!(
            "cells_share_connectivity: const mismatch on pin {}: P {:?} vs D {:?}",
            pin_idx, p_src, d_src
        );
    }
    ok
}

fn source_matches_mapped_io<'p, 'd>(
    pin_idx: usize,
    p_src_cell: prjunnamed_netlist::CellRef<'p>,
    p_bit: usize,
    d_src: &Source<'d>,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    let Some(d_src_cell) = mapping.get_design_cell(p_src_cell) else {
        // Unmapped pattern source; unconstrained at this stage.
        return true;
    };

    // For mapped IO, accept either Io or Gate on the design side (same cell + bit).
    let ok = match d_src {
        Source::Io(d_cell, d_bit) => *d_cell == d_src_cell && *d_bit == p_bit,
        Source::Gate(d_cell, d_bit) => *d_cell == d_src_cell && *d_bit == p_bit,
        Source::Const(_) => false,
    };

    if !ok {
        trace!(
            "cells_share_connectivity: fanin mismatch on pin {}: expected mapped {:?} -> {:?}, got {:?}",
            pin_idx,
            Source::Io(p_src_cell, p_bit),
            d_src_cell.debug_index(),
            d_src
        );
    }

    ok
}

fn source_matches_mapped_gate<'p, 'd>(
    pin_idx: usize,
    p_src_cell: prjunnamed_netlist::CellRef<'p>,
    p_bit: usize,
    d_src: &Source<'d>,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    let Some(d_src_cell) = mapping.get_design_cell(p_src_cell) else {
        // Unmapped pattern source; unconstrained at this stage.
        return true;
    };

    // For mapped Gate, require Gate on the design side (same cell + bit).
    let ok =
        matches!(d_src, Source::Gate(d_cell, d_bit) if *d_cell == d_src_cell && *d_bit == p_bit);

    if !ok {
        trace!(
            "cells_share_connectivity: fanin mismatch on pin {}: expected mapped gate {:?} -> {:?}, got {:?}",
            pin_idx,
            Source::Gate(p_src_cell, p_bit),
            d_src_cell.debug_index(),
            d_src
        );
    }

    ok
}

fn pin_sources_compatible<'p, 'd>(
    pin_idx: usize,
    p_src: &Source<'p>,
    d_src: &Source<'d>,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    match p_src {
        Source::Const(_) => source_matches_const(pin_idx, p_src, d_src),
        Source::Io(p_src_cell, p_bit) => {
            source_matches_mapped_io(pin_idx, *p_src_cell, *p_bit, d_src, mapping)
        }
        Source::Gate(p_src_cell, p_bit) => {
            source_matches_mapped_gate(pin_idx, *p_src_cell, *p_bit, d_src, mapping)
        }
    }
}

fn check_fanin<'p, 'd>(
    p_cell: &CellWrapper<'p>,
    d_cell: &CellWrapper<'d>,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    p_cell.pins().iter().enumerate().all(|(pin_idx, p_src)| {
        let Some(d_src) = d_cell.pins().get(pin_idx) else {
            trace!(
                "cells_share_connectivity: P {} pin {} has no corresponding D pin",
                p_cell.summary(),
                pin_idx
            );
            return false;
        };
        pin_sources_compatible(pin_idx, p_src, d_src, mapping)
    })
}

fn fanout_edge_ok<'d>(
    d_fanouts: &[(prjunnamed_netlist::CellRef<'d>, usize)],
    d_sink_cell: prjunnamed_netlist::CellRef<'d>,
    pin_idx: usize,
) -> bool {
    let sink_commutative = CellKind::from(d_sink_cell.get().as_ref()).is_commutative_inputs();

    let ok = if sink_commutative {
        d_fanouts.iter().any(|(s, _)| *s == d_sink_cell)
    } else {
        d_fanouts
            .iter()
            .any(|(s, i)| *s == d_sink_cell && *i == pin_idx)
    };

    trace!(
        "cells_share_connectivity: check mapped sink D#{} @pin={} (commutative={}) -> {}",
        d_sink_cell.debug_index(),
        pin_idx,
        sink_commutative,
        ok
    );

    ok
}

fn check_fanout<'p, 'd>(
    p_cell: &CellWrapper<'p>,
    d_cell: &CellWrapper<'d>,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    let d_fanouts = d_index.get_fanouts(d_cell.cref());
    let p_fanouts = p_index.get_fanouts(p_cell.cref());

    trace!(
        "cells_share_connectivity: P {} fanouts={} | D {} fanouts={}",
        p_cell.summary(),
        p_fanouts.len(),
        d_cell.summary(),
        d_fanouts.len()
    );

    p_fanouts
        .iter()
        .filter_map(|(p_sink_cell, pin_idx)| {
            mapping
                .get_design_cell(*p_sink_cell)
                .map(|d_sink_cell| (d_sink_cell, *pin_idx))
        })
        .all(|(d_sink_cell, pin_idx)| fanout_edge_ok(d_fanouts, d_sink_cell, pin_idx))
}

fn cells_share_connectivity<'p, 'd>(
    p_cell: &CellWrapper<'p>,
    d_cell: &CellWrapper<'d>,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    _config: &Config,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    if !check_fanin(p_cell, d_cell, mapping) {
        trace!(
            "cells_share_connectivity: P {} vs D {} -> fanin check FAILED",
            p_cell.summary(),
            d_cell.summary()
        );
        return false;
    }

    let result = check_fanout(p_cell, d_cell, p_index, d_index, mapping);

    trace!(
        "cells_share_connectivity: P {} vs D {} -> {}",
        p_cell.summary(),
        d_cell.summary(),
        result
    );

    result
}
