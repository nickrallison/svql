#![allow(dead_code)]
mod graph_index;
mod isomorphism;
mod node;

use graph_index::GraphIndex;
use isomorphism::NodeMapping;
use prjunnamed_netlist::{CellRef, Design};

use std::collections::{HashMap, HashSet, VecDeque};
use svql_common::{Config, DedupeMode};

use crate::node::{NodeSource, NodeType};

pub use prjunnamed_netlist::CellRef as NodeWrapper;

#[derive(Clone, Debug, Default)]
pub struct SubgraphIsomorphism<'p, 'd> {
    // Mapping of pattern nodes to design nodes (and reverse)
    mapping: NodeMapping<'p, 'd>,

    // Boundary IO lookup tables
    pub input_by_name: HashMap<&'p str, CellRef<'p>>,
    pub output_by_name: HashMap<&'p str, CellRef<'p>>,
}

impl<'p, 'd> SubgraphIsomorphism<'p, 'd> {
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
        let d_src = self.mapping.get_design_node(*p_input)?;
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

pub fn find_subgraph_isomorphisms<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    tracing::event!(
        tracing::Level::TRACE,
        "find_subgraph_isomorphisms: start. pattern cells={} design cells={}",
        pattern.iter_cells().count(),
        design.iter_cells().count()
    );

    let pattern_index = GraphIndex::build(pattern);
    let design_index = GraphIndex::build(design);

    tracing::event!(
        tracing::Level::TRACE,
        "find_subgraph_isomorphisms: node counts: pattern={} design={}",
        pattern_index.node_count(),
        design_index.node_count()
    );

    if pattern_index.node_count() == 0 || design_index.node_count() == 0 {
        tracing::event!(
            tracing::Level::TRACE,
            "find_subgraph_isomorphisms: early return (empty node count)"
        );
        return Vec::new();
    }

    // Build boundary IO maps for the pattern (by name).
    let input_by_name: HashMap<&'p str, CellRef<'p>> = pattern_index
        .get_nodes_topo()
        .iter()
        .filter(|c| matches!(pattern_index.get_node_type(**c), NodeType::Input))
        .filter_map(|c| pattern_index.get_input_name(*c).map(|n| (n, *c)))
        .collect();

    let output_by_name: HashMap<&'p str, CellRef<'p>> = pattern_index
        .get_nodes_topo()
        .iter()
        .filter(|c| matches!(pattern_index.get_node_type(**c), NodeType::Output))
        .filter_map(|c| pattern_index.get_output_name(*c).map(|n| (n, *c)))
        .collect();

    // in topological_order, only gates & inputs
    let pattern_mapping_queue: VecDeque<CellRef<'p>> = pattern_index
        .get_nodes_topo()
        .iter()
        .filter(|c| !matches!(pattern_index.get_node_type(**c), NodeType::Output))
        .copied()
        .collect();

    tracing::event!(
        tracing::Level::TRACE,
        "find_subgraph_isomorphisms: initial pattern mapping queue size={}",
        pattern_mapping_queue.len()
    );

    let initial_node_mapping: NodeMapping<'p, 'd> = NodeMapping::new();

    tracing::event!(
        tracing::Level::INFO,
        "find_subgraph_isomorphisms: executing recursive search"
    );

    let results = find_isomorphisms_recursive(
        &pattern_index,
        &design_index,
        config,
        initial_node_mapping,
        pattern_mapping_queue,
        &input_by_name,
        &output_by_name,
        0, // depth
    );

    tracing::event!(
        tracing::Level::INFO,
        "find_subgraph_isomorphisms: results={} unique_sigs={:?}",
        results.len(),
        results
            .iter()
            .map(|m| m.mapping.signature())
            .collect::<Vec<_>>()
    );

    results
}

// -------------
// New helper: narrow candidates for pattern Inputs using mapped neighbors.
// -------------
fn find_candidate_drivers_for_pattern_input<'p, 'd>(
    current: CellRef<'p>,
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> Option<Vec<CellRef<'d>>> {
    // For each mapped fanout sink, gather its possible driver(s), then intersect across sinks.
    let mapped_sinks: Vec<(CellRef<'p>, usize, CellRef<'d>)> = pattern_index
        .get_fanouts(current)
        .iter()
        .filter_map(|(p_sink_node, pin_idx)| {
            mapping
                .get_design_node(*p_sink_node)
                .map(|d_sink_node| (*p_sink_node, *pin_idx, d_sink_node))
        })
        .collect();

    if mapped_sinks.is_empty() {
        return None;
    }

    let mut sets: Vec<Vec<CellRef<'d>>> = mapped_sinks
        .iter()
        .map(|(_p_sink, pin_idx, d_sink)| {
            let sink_type = design_index.get_node_type(*d_sink);

            if sink_type.has_commutative_inputs() {
                // Either input pin acceptable: take drivers of all pins
                design_index.drivers_of_sink_all_pins(*d_sink)
            } else {
                // Specific pin must match
                design_index
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

fn find_isomorphisms_recursive<'p, 'd>(
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    config: &Config,
    node_mapping: NodeMapping<'p, 'd>,
    mut pattern_mapping_queue: VecDeque<CellRef<'p>>,
    input_by_name: &HashMap<&'p str, CellRef<'p>>,
    output_by_name: &HashMap<&'p str, CellRef<'p>>,
    depth: usize,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    let Some(current) = pattern_mapping_queue.pop_front() else {
        tracing::event!(
            tracing::Level::TRACE,
            "find_isomorphisms_recursive[depth={}]: base case reached. mapping size={}",
            depth,
            node_mapping.len()
        );

        return vec![SubgraphIsomorphism {
            mapping: node_mapping,
            input_by_name: input_by_name.clone(),
            output_by_name: output_by_name.clone(),
        }];
    };

    tracing::event!(
        tracing::Level::TRACE,
        "find_isomorphisms_recursive[depth={}]: current=#{} type={:?} | remaining_queue={} | mapping_size={}",
        depth,
        current.debug_index(),
        pattern_index.get_node_type(current),
        pattern_mapping_queue.len(),
        node_mapping.len()
    );

    let current_type = pattern_index.get_node_type(current);

    // Narrowing Candidates for next level of recursion
    let candidates: Vec<CellRef<'d>> = match current_type {
        NodeType::Input => find_candidate_drivers_for_pattern_input(
            current,
            pattern_index,
            design_index,
            &node_mapping,
        )
        .unwrap_or_default(),
        _ => design_index.get_by_type(current_type).to_vec(),
    };

    // Function to be executed on each possible candidate
    let process_candidate = |d_node: CellRef<'d>| -> Vec<SubgraphIsomorphism<'p, 'd>> {
        if is_node_already_mapped(d_node, &node_mapping, depth) {
            return Vec::new();
        }
        if !are_nodes_compatible(current_type, design_index.get_node_type(d_node)) {
            return Vec::new();
        }
        if !is_node_connectivity_valid(
            current,
            d_node,
            pattern_index,
            design_index,
            config,
            &node_mapping,
        ) {
            return Vec::new();
        }

        // Accepted candidate; recurse immediately.
        let mut new_node_mapping = node_mapping.clone();
        new_node_mapping.insert(current, d_node);

        find_isomorphisms_recursive(
            pattern_index,
            design_index,
            config,
            new_node_mapping,
            pattern_mapping_queue.clone(),
            input_by_name,
            output_by_name,
            depth + 1,
        )
    };

    let mut results: Vec<SubgraphIsomorphism<'p, 'd>> = candidates
        .into_iter()
        .flat_map(|d_node| process_candidate(d_node))
        .collect();

    if matches!(config.dedupe, DedupeMode::AutoMorph) {
        let mut seen: HashSet<Vec<usize>> = HashSet::new();
        results.retain(|m| seen.insert(m.mapping.signature()));
        tracing::event!(
            tracing::Level::TRACE,
            "find_isomorphisms_recursive[depth={}]: after dedupe -> {}",
            depth,
            results.len()
        );
    }

    results
}

fn is_node_already_mapped(
    d_node: CellRef<'_>,
    node_mapping: &NodeMapping<'_, '_>,
    depth: usize,
) -> bool {
    if node_mapping.design_mapping().contains_key(&d_node) {
        tracing::event!(
            tracing::Level::TRACE,
            "find_isomorphisms_recursive[depth={}]: skip D #{} (already mapped)",
            depth,
            d_node.debug_index()
        );
        return true;
    }
    false
}

fn are_nodes_compatible(p_type: NodeType, d_type: NodeType) -> bool {
    if matches!(p_type, NodeType::Input) {
        // Inputs can map to any node; fanin/fanout checks will constrain sufficiently.
        return true;
    }
    if p_type != d_type {
        tracing::event!(
            tracing::Level::TRACE,
            "nodes incompatible: type mismatch P {:?} vs D {:?}",
            p_type,
            d_type
        );
        return false;
    }
    true
}

fn source_matches_const<'d>(
    pin_idx: usize,
    p_src: &NodeSource<'_>,
    d_src: &NodeSource<'d>,
) -> bool {
    let ok = matches!(d_src, NodeSource::Const(dt) if matches!(p_src, NodeSource::Const(pt) if dt == pt));
    if !ok {
        tracing::event!(
            tracing::Level::TRACE,
            "is_node_connectivity_valid: const mismatch on pin {}: P {:?} vs D {:?}",
            pin_idx,
            p_src,
            d_src
        );
    }
    ok
}

fn source_matches_mapped_io<'p, 'd>(
    pin_idx: usize,
    p_src_node: prjunnamed_netlist::CellRef<'p>,
    p_bit: usize,
    d_src: &NodeSource<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> bool {
    let Some(d_src_node) = mapping.get_design_node(p_src_node) else {
        // Unmapped pattern source; unconstrained at this stage.
        return true;
    };

    let ok = match d_src {
        NodeSource::Io(d_node, d_bit) => *d_node == d_src_node && *d_bit == p_bit,
        NodeSource::Gate(d_node, d_bit) => *d_node == d_src_node && *d_bit == p_bit,
        NodeSource::Const(_) => false,
    };

    if !ok {
        tracing::event!(
            tracing::Level::TRACE,
            "is_node_connectivity_valid: fanin mismatch on pin {}: expected mapped {:?} -> {:?}, got {:?}",
            pin_idx,
            NodeSource::Io(p_src_node, p_bit),
            d_src_node.debug_index(),
            d_src
        );
    }

    ok
}

fn source_matches_mapped_gate<'p, 'd>(
    pin_idx: usize,
    p_src_node: prjunnamed_netlist::CellRef<'p>,
    p_bit: usize,
    d_src: &NodeSource<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> bool {
    let Some(d_src_node) = mapping.get_design_node(p_src_node) else {
        // Unmapped pattern source; unconstrained at this stage.
        return true;
    };

    let ok = matches!(d_src, NodeSource::Gate(d_node, d_bit) if *d_node == d_src_node && *d_bit == p_bit);

    if !ok {
        tracing::event!(
            tracing::Level::TRACE,
            "is_node_connectivity_valid: fanin mismatch on pin {}: expected mapped gate {:?} -> {:?}, got {:?}",
            pin_idx,
            NodeSource::Gate(p_src_node, p_bit),
            d_src_node.debug_index(),
            d_src
        );
    }

    ok
}

fn pin_sources_compatible<'p, 'd>(
    pin_idx: usize,
    p_src: &NodeSource<'p>,
    d_src: &NodeSource<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> bool {
    match p_src {
        NodeSource::Const(_) => source_matches_const(pin_idx, p_src, d_src),
        NodeSource::Io(p_src_node, p_bit) => {
            source_matches_mapped_io(pin_idx, *p_src_node, *p_bit, d_src, mapping)
        }
        NodeSource::Gate(p_src_node, p_bit) => {
            source_matches_mapped_gate(pin_idx, *p_src_node, *p_bit, d_src, mapping)
        }
    }
}

fn validate_fanin_connections<'p, 'd>(
    p_node: CellRef<'p>,
    d_node: CellRef<'d>,
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> bool {
    let p_sources = pattern_index.get_node_sources(p_node);
    let d_sources = design_index.get_node_sources(d_node);

    p_sources.iter().enumerate().all(|(pin_idx, p_src)| {
        let Some(d_src) = d_sources.get(pin_idx) else {
            tracing::event!(
                tracing::Level::TRACE,
                "is_node_connectivity_valid: P {} pin {} has no corresponding D pin",
                p_node.debug_index(),
                pin_idx
            );
            return false;
        };
        pin_sources_compatible(pin_idx, p_src, d_src, mapping)
    })
}

// Use O(1) membership via design_index instead of scanning a Vec
fn fanout_edge_ok<'d>(
    design_index: &GraphIndex<'d>,
    d_driver: prjunnamed_netlist::CellRef<'d>,
    d_sink_node: prjunnamed_netlist::CellRef<'d>,
    pin_idx: usize,
) -> bool {
    let sink_commutative = design_index
        .get_node_type(d_sink_node)
        .has_commutative_inputs();

    let ok = if sink_commutative {
        design_index.has_fanout_to(d_driver, d_sink_node)
    } else {
        design_index.has_fanout_to_pin(d_driver, d_sink_node, pin_idx)
    };

    tracing::event!(
        tracing::Level::TRACE,
        "is_node_connectivity_valid: check mapped sink D#{} @pin={} (commutative={}) -> {}",
        d_sink_node.debug_index(),
        pin_idx,
        sink_commutative,
        ok
    );

    ok
}

fn validate_fanout_connections<'p, 'd>(
    p_node: CellRef<'p>,
    d_node: CellRef<'d>,
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> bool {
    let p_fanouts = pattern_index.get_fanouts(p_node);

    // Only need to validate edges to already-mapped sinks.
    p_fanouts
        .iter()
        .filter_map(|(p_sink_node, pin_idx)| {
            mapping
                .get_design_node(*p_sink_node)
                .map(|d_sink_node| (d_sink_node, *pin_idx))
        })
        .all(|(d_sink_node, pin_idx)| fanout_edge_ok(design_index, d_node, d_sink_node, pin_idx))
}

fn is_node_connectivity_valid<'p, 'd>(
    p_node: CellRef<'p>,
    d_node: CellRef<'d>,
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    _config: &Config,
    mapping: &NodeMapping<'p, 'd>,
) -> bool {
    let valid_fanin =
        validate_fanin_connections(p_node, d_node, pattern_index, design_index, mapping);
    if !valid_fanin {
        tracing::event!(
            tracing::Level::TRACE,
            "is_node_connectivity_valid: P #{} vs D #{} -> fanin check FAILED",
            p_node.debug_index(),
            d_node.debug_index(),
        );

        return false;
    }

    let valid_fanout =
        validate_fanout_connections(p_node, d_node, pattern_index, design_index, mapping);
    if !valid_fanout {
        tracing::event!(
            tracing::Level::TRACE,
            "is_node_connectivity_valid: P #{} vs D #{} -> fanout check FAILED",
            p_node.debug_index(),
            d_node.debug_index(),
        );

        return false;
    }

    tracing::event!(
        tracing::Level::TRACE,
        "is_node_connectivity_valid: P #{} vs D #{} -> {}",
        p_node.debug_index(),
        d_node.debug_index(),
        true
    );

    true
}
