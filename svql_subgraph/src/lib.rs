mod cell;
mod index;
mod mapping;

use index::Index;
use mapping::CellMapping;
use prjunnamed_netlist::{CellRef, Design};

use std::collections::{HashMap, VecDeque};
use svql_common::{Config, DedupeMode};
use tracing::{Level, info, trace};

use crate::cell::{CellKind, Source};

pub use prjunnamed_netlist::CellRef as CellWrapper;

#[derive(Clone, Debug, Default)]
pub struct SubgraphMatch<'p, 'd> {
    // Mapping of pattern cells to design cells (and reverse)
    mapping: CellMapping<'p, 'd>,

    // Boundary IO lookup tables
    pub input_by_name: HashMap<&'p str, CellRef<'p>>,
    pub output_by_name: HashMap<&'p str, CellRef<'p>>,
}

impl<'p, 'd> SubgraphMatch<'p, 'd> {
    pub fn len(&self) -> usize {
        self.mapping.len()
    }
    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub fn design_source_of_input_bit(
        &self,
        name: &str,
        bit: usize,
    ) -> Option<(CellRef<'d>, usize)> {
        let p_input = self.input_by_name.get(name)?;
        let d_src = self.mapping.get_design_cell(*p_input)?;
        Some((d_src, bit))
    }

    pub fn design_driver_of_output_bit(
        &self,
        _name: &str,
        _bit: usize,
    ) -> Option<(CellRef<'d>, usize)> {
        // Not implemented yet; left untouched for now.
        None
    }
}

pub fn find_subgraphs<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> Vec<SubgraphMatch<'p, 'd>> {
    if tracing::level_enabled!(Level::TRACE) {
        tracing::event!(tracing::Level::TRACE, 
            "find_subgraphs: start. pattern cells={} design cells={}",
            pattern.iter_cells().count(),
            design.iter_cells().count()
        );
    }

    let p_index = Index::build(pattern);
    let d_index = Index::build(design);

    if tracing::level_enabled!(Level::TRACE) {
        tracing::event!(tracing::Level::TRACE, 
            "find_subgraphs: gate counts: pattern={} design={}",
            p_index.gate_count(),
            d_index.gate_count()
        );
    }

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        tracing::event!(tracing::Level::TRACE, "find_subgraphs: early return (empty gate count)");
        return Vec::new();
    }

    // Build boundary IO maps for the pattern (by name).
    let input_by_name: HashMap<&'p str, CellRef<'p>> = p_index
        .get_cells_topo()
        .iter()
        .filter(|c| matches!(p_index.get_cell_kind(**c), CellKind::Input))
        .filter_map(|c| p_index.get_cell_input_name(*c).map(|n| (n, *c)))
        .collect();

    let output_by_name: HashMap<&'p str, CellRef<'p>> = p_index
        .get_cells_topo()
        .iter()
        .filter(|c| matches!(p_index.get_cell_kind(**c), CellKind::Output))
        .filter_map(|c| p_index.get_cell_output_name(*c).map(|n| (n, *c)))
        .collect();

    // in topological_order, only gates & inputs
    let p_mapping_queue: VecDeque<CellRef<'p>> = p_index
        .get_cells_topo()
        .into_iter()
        .filter(|c| !matches!(p_index.get_cell_kind(**c), CellKind::Output))
        .map(|c| *c)
        .collect();

    if tracing::level_enabled!(Level::TRACE) {
        tracing::event!(tracing::Level::TRACE, 
            "find_subgraphs: initial pattern mapping queue size={}",
            p_mapping_queue.len()
        );
    }

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

    tracing::event!(tracing::Level::INFO, 
        "find_subgraphs: results={} unique_sigs={:?}",
        results.len(),
        results.iter().map(|m| m.mapping.sig()).collect::<Vec<_>>()
    );

    results
}

// -------------
// New helper: narrow candidates for pattern Inputs using mapped neighbors.
// -------------
fn candidate_drivers_for_pattern_input<'p, 'd>(
    current: CellRef<'p>,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    mapping: &CellMapping<'p, 'd>,
) -> Option<Vec<CellRef<'d>>> {
    // For each mapped fanout sink, gather its possible driver(s), then intersect across sinks.
    let mapped_sinks: Vec<(CellRef<'p>, usize, CellRef<'d>)> = p_index
        .get_fanouts(current)
        .iter()
        .filter_map(|(p_sink_cell, pin_idx)| {
            mapping
                .get_design_cell(*p_sink_cell)
                .map(|d_sink_cell| (*p_sink_cell, *pin_idx, d_sink_cell))
        })
        .collect();

    if mapped_sinks.is_empty() {
        return None;
    }

    let mut sets: Vec<Vec<CellRef<'d>>> = mapped_sinks
        .iter()
        .map(|(_p_sink, pin_idx, d_sink)| {
            let sink_kind = d_index.get_cell_kind(*d_sink);

            if sink_kind.is_commutative_inputs() {
                // Either input pin acceptable: take drivers of all pins
                d_index.drivers_of_sink_all_pins(*d_sink)
            } else {
                // Specific pin must match
                d_index
                    .driver_of_sink_pin(*d_sink, *pin_idx)
                    .into_iter()
                    .collect()
            }
        })
        .filter(|v| !v.is_empty())
        .collect();

    if sets.is_empty() {
        return None;
    }

    // Intersect all sets
    let mut acc = sets.remove(0);
    acc.sort_by_key(|c| c.debug_index());
    acc.dedup();

    for s in sets {
        let mut next: Vec<CellRef<'d>> = Vec::with_capacity(acc.len().min(s.len()));
        for c in acc.into_iter() {
            if s.contains(&c) {
                next.push(c);
            }
        }
        acc = next;
        if acc.is_empty() {
            break;
        }
    }

    if acc.is_empty() { None } else { Some(acc) }
}

fn find_subgraphs_recursive<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    config: &Config,
    cell_mapping: CellMapping<'p, 'd>,
    mut p_mapping_queue: VecDeque<CellRef<'p>>,
    input_by_name: &HashMap<&'p str, CellRef<'p>>,
    output_by_name: &HashMap<&'p str, CellRef<'p>>,
    depth: usize,
) -> Vec<SubgraphMatch<'p, 'd>> {
    let Some(current) = p_mapping_queue.pop_front() else {
        if tracing::level_enabled!(Level::TRACE) {
            tracing::event!(tracing::Level::TRACE, 
                "find_subgraphs_recursive[depth={}]: base case reached. mapping size={}",
                depth,
                cell_mapping.len()
            );
        }
        return vec![SubgraphMatch {
            mapping: cell_mapping,
            input_by_name: input_by_name.clone(),
            output_by_name: output_by_name.clone(),
        }];
    };

    if tracing::level_enabled!(Level::TRACE) {
        tracing::event!(tracing::Level::TRACE, 
            "find_subgraphs_recursive[depth={}]: current=#{} kind={:?} | remaining_queue={} | mapping_size={}",
            depth,
            current.debug_index(),
            p_index.get_cell_kind(current),
            p_mapping_queue.len(),
            cell_mapping.len()
        );
    }

    let mut total_candidates = 0usize;
    let mut already_mapped = 0usize;
    let mut incompatible = 0usize;
    let mut connectivity_fail = 0usize;

    let current_kind = p_index.get_cell_kind(current);

    // New: drastically prune candidates for pattern Inputs using mapped fanout neighbors
    let narrowed_candidates = if matches!(current_kind, CellKind::Input) {
        candidate_drivers_for_pattern_input(current, p_index, d_index, &cell_mapping)
    } else {
        None
    };

    let d_candidates: Vec<CellRef<'d>> = match narrowed_candidates {
        Some(vec) if !vec.is_empty() => vec,
        _ => {
            if matches!(current_kind, CellKind::Input) {
                // Worst case fallback: all nodes (previous behavior)
                d_index.get_cells_topo().iter().map(|c| *c).collect()
            } else {
                d_index
                    .get_by_kind(current_kind)
                    .iter()
                    .map(|c| *c)
                    .collect()
            }
        }
    };

    total_candidates = d_candidates.len();

    let new_cell_mappings: Vec<CellMapping<'p, 'd>> = d_candidates
        .into_iter()
        .filter(|d_cell| !d_cell_already_mapped(*d_cell, &cell_mapping, depth, &mut already_mapped))
        .filter(|d_cell| {
            d_cell_compatible(
                current_kind,
                d_index.get_cell_kind(*d_cell),
                &mut incompatible,
            )
        })
        .filter(|d_cell| {
            d_cell_valid_connectivity(
                current,
                *d_cell,
                p_index,
                d_index,
                config,
                &cell_mapping,
                &mut connectivity_fail,
            )
        })
        .map(|d_cell| {
            let mut new_mapping = cell_mapping.clone();
            new_mapping.insert(current, d_cell);
            new_mapping
        })
        .collect();

    if tracing::level_enabled!(Level::TRACE) {
        tracing::event!(tracing::Level::TRACE, 
            "find_subgraphs_recursive[depth={}]: candidate stats: total={} already_mapped={} incompatible={} connectivity_fail={} accepted={}",
            depth,
            total_candidates,
            already_mapped,
            incompatible,
            connectivity_fail,
            new_cell_mappings.len()
        );
    }

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

        if tracing::level_enabled!(Level::TRACE) {
            tracing::event!(tracing::Level::TRACE, 
                "find_subgraphs_recursive[depth={}]: results before_dedup={} after_dedup={}",
                depth, before_dedup, after_dedup
            );
        }
    }

    results
}

fn d_cell_already_mapped(
    d_cell: CellRef<'_>,
    cell_mapping: &CellMapping<'_, '_>,
    depth: usize,
    already_mapped: &mut usize,
) -> bool {
    if cell_mapping.design_mapping().contains_key(&d_cell) {
        *already_mapped += 1;
        tracing::event!(tracing::Level::TRACE, 
            "find_subgraphs_recursive[depth={}]: skip D #{} (already mapped)",
            depth,
            d_cell.debug_index()
        );
        return true;
    }
    false
}

fn d_cell_compatible(p_kind: CellKind, d_kind: CellKind, incompatibility: &mut usize) -> bool {
    if matches!(p_kind, CellKind::Input) {
        // Inputs can map to any node; fanin/fanout checks will constrain sufficiently.
        return true;
    }
    if p_kind != d_kind {
        *incompatibility += 1;
        tracing::event!(tracing::Level::TRACE, 
            "cells incompatible: kind mismatch P {:?} vs D {:?}",
            p_kind, d_kind
        );
        return false;
    }
    true
}

fn source_matches_const<'d>(pin_idx: usize, p_src: &Source<'_>, d_src: &Source<'d>) -> bool {
    let ok = matches!(d_src, Source::Const(dt) if matches!(p_src, Source::Const(pt) if dt == pt));
    if !ok {
        tracing::event!(tracing::Level::TRACE, 
            "d_cell_valid_connectivity: const mismatch on pin {}: P {:?} vs D {:?}",
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

    let ok = match d_src {
        Source::Io(d_cell, d_bit) => *d_cell == d_src_cell && *d_bit == p_bit,
        Source::Gate(d_cell, d_bit) => *d_cell == d_src_cell && *d_bit == p_bit,
        Source::Const(_) => false,
    };

    if !ok {
        tracing::event!(tracing::Level::TRACE, 
            "d_cell_valid_connectivity: fanin mismatch on pin {}: expected mapped {:?} -> {:?}, got {:?}",
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

    let ok =
        matches!(d_src, Source::Gate(d_cell, d_bit) if *d_cell == d_src_cell && *d_bit == p_bit);

    if !ok {
        tracing::event!(tracing::Level::TRACE, 
            "d_cell_valid_connectivity: fanin mismatch on pin {}: expected mapped gate {:?} -> {:?}, got {:?}",
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
    p_cell: CellRef<'p>,
    d_cell: CellRef<'d>,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    let p_sources = p_index.get_cell_sources(p_cell);
    let d_sources = d_index.get_cell_sources(d_cell);

    p_sources.iter().enumerate().all(|(pin_idx, p_src)| {
        let Some(d_src) = d_sources.get(pin_idx) else {
            tracing::event!(tracing::Level::TRACE, 
                "d_cell_valid_connectivity: P {} pin {} has no corresponding D pin",
                p_cell.debug_index(),
                pin_idx
            );
            return false;
        };
        pin_sources_compatible(pin_idx, p_src, d_src, mapping)
    })
}

// Changed: use O(1) membership via d_index instead of scanning a Vec
fn fanout_edge_ok<'d>(
    d_index: &Index<'d>,
    d_driver: prjunnamed_netlist::CellRef<'d>,
    d_sink_cell: prjunnamed_netlist::CellRef<'d>,
    pin_idx: usize,
) -> bool {
    let sink_commutative = d_index.get_cell_kind(d_sink_cell).is_commutative_inputs();

    let ok = if sink_commutative {
        d_index.has_fanout_to(d_driver, d_sink_cell)
    } else {
        d_index.has_fanout_to_pin(d_driver, d_sink_cell, pin_idx)
    };

    tracing::event!(tracing::Level::TRACE, 
        "d_cell_valid_connectivity: check mapped sink D#{} @pin={} (commutative={}) -> {}",
        d_sink_cell.debug_index(),
        pin_idx,
        sink_commutative,
        ok
    );

    ok
}

fn check_fanout<'p, 'd>(
    p_cell: CellRef<'p>,
    d_cell: CellRef<'d>,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    mapping: &CellMapping<'p, 'd>,
) -> bool {
    let p_fanouts = p_index.get_fanouts(p_cell);

    // Only need to validate edges to already-mapped sinks.
    p_fanouts
        .iter()
        .filter_map(|(p_sink_cell, pin_idx)| {
            mapping
                .get_design_cell(*p_sink_cell)
                .map(|d_sink_cell| (d_sink_cell, *pin_idx))
        })
        .all(|(d_sink_cell, pin_idx)| fanout_edge_ok(d_index, d_cell, d_sink_cell, pin_idx))
}

fn d_cell_valid_connectivity<'p, 'd>(
    p_cell: CellRef<'p>,
    d_cell: CellRef<'d>,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    _config: &Config,
    mapping: &CellMapping<'p, 'd>,
    invalid_connectivity: &mut usize,
) -> bool {
    let valid_fanin = check_fanin(p_cell, d_cell, p_index, d_index, mapping);
    if !valid_fanin {
        if tracing::level_enabled!(Level::TRACE) {
            tracing::event!(tracing::Level::TRACE, 
                "d_cell_valid_connectivity: P #{} vs D #{} -> fanin check FAILED",
                p_cell.debug_index(),
                d_cell.debug_index(),
            );
        }
        *invalid_connectivity += 1;
        return false;
    }

    let valid_fanout = check_fanout(p_cell, d_cell, p_index, d_index, mapping);
    if !valid_fanout {
        if tracing::level_enabled!(Level::TRACE) {
            tracing::event!(tracing::Level::TRACE, 
                "d_cell_valid_connectivity: P #{} vs D #{} -> fanout check FAILED",
                p_cell.debug_index(),
                d_cell.debug_index(),
            );
        }
        *invalid_connectivity += 1;
        return false;
    }

    tracing::event!(tracing::Level::TRACE, 
        "d_cell_valid_connectivity: P #{} vs D #{} -> {}",
        p_cell.debug_index(),
        d_cell.debug_index(),
        true
    );

    true
}
