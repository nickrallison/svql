#![allow(dead_code)]
mod graph_index;
mod isomorphism;
mod node;

use graph_index::GraphIndex;
use isomorphism::NodeMapping;
use prjunnamed_netlist::{CellRef, Design};

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
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

    let pattern_mapping_queue: VecDeque<CellRef<'p>> = {
        // in topological_order, only gates & inputs
        let mut initial_pattern_mapping: Vec<CellRef<'p>> = pattern_index
            .get_nodes_topo()
            .iter()
            .filter(|c| !matches!(pattern_index.get_node_type(**c), NodeType::Output))
            .copied()
            .collect();

        // stable sort inputs to back
        initial_pattern_mapping.sort_by(|a, b| {
            let a_is_input = matches!(pattern_index.get_node_type(*a), NodeType::Input);
            let b_is_input = matches!(pattern_index.get_node_type(*b), NodeType::Input);
            a_is_input.cmp(&b_is_input)
        });

        initial_pattern_mapping.into()
    };

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

/// Base candidate set from node type only:
/// - For pattern Input nodes: all design nodes (Name already excluded by GraphIndex)
/// - For others: all design nodes of that NodeType
fn initial_candidates_for_type<'d>(
    design_index: &GraphIndex<'d>,
    current_type: NodeType,
) -> HashSet<CellRef<'d>> {
    match current_type {
        NodeType::Input => design_index.get_nodes_topo().iter().copied().collect(),
        _ => design_index
            .get_by_type(current_type)
            .iter()
            .copied()
            .collect(),
    }
}

fn intersect<T>(sets: impl IntoIterator<Item = HashSet<T>>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
{
    let mut iter = sets.into_iter();
    let mut acc = match iter.next() {
        Some(set) => set,
        None => return HashSet::new(),
    };
    for set in iter {
        acc = acc.intersection(&set).cloned().collect();
    }
    acc
}

/// Candidates restricted by already-mapped sinks (fan-out constraints).
/// For each mapped sink of the pattern node, compute the set of possible drivers
/// in the design and intersect across sinks. If none of the sinks are mapped,
/// returns None (no restriction).
fn candidates_from_mapped_sinks<'p, 'd>(
    pattern_current: CellRef<'p>,
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> Option<HashSet<CellRef<'d>>> {
    // For each mapped fanout sink, gather its possible driver(s), then intersect across sinks.
    let mapped_sinks: Vec<(CellRef<'p>, usize, CellRef<'d>)> = pattern_index
        .get_fanouts(pattern_current)
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

    let sets = mapped_sinks
        .iter()
        .map(|(_p_sink, pin_idx, d_sink)| {
            let sink_type = design_index.get_node_type(*d_sink);

            if sink_type.has_commutative_inputs() {
                // Any driver to any pin
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
        .map(|v| v.into_iter().collect::<HashSet<CellRef<'d>>>());

    Some(intersect(sets))
}

/// Candidates restricted by already-mapped sources (fan-in constraints).
/// For each mapped source of the pattern node, collect the sinks in the design
/// that are driven by the corresponding mapped design driver, respecting
/// commutativity of the current node. Intersect across all mapped sources.
/// If no sources are mapped, returns None (no restriction).
fn candidates_from_mapped_sources<'p, 'd>(
    pattern_current: CellRef<'p>,
    current_type: NodeType,
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> Option<HashSet<CellRef<'d>>> {
    let commutative = current_type.has_commutative_inputs();

    let mapped_sources: Vec<(usize, NodeSource<'p>)> = pattern_index
        .get_node_sources(pattern_current)
        .iter()
        .cloned()
        .enumerate()
        .collect();

    if mapped_sources.is_empty() {
        return None;
    }

    let sets = mapped_sources
        .into_iter()
        .filter_map(|(pin_idx, p_src)| match p_src {
            NodeSource::Gate(p_src_node, _pbit) | NodeSource::Io(p_src_node, _pbit) => mapping
                .get_design_node(p_src_node)
                .map(|d_src_node| (pin_idx, d_src_node)),
            NodeSource::Const(_) => None, // leave const handling to full connectivity validation
        })
        .map(|(pin_idx, d_src_node)| {
            // For the mapped source driver, get all its fanouts in the design.
            // If commutative, any pin is acceptable; otherwise, the exact pin must match.
            let fanouts = design_index.get_fanouts(d_src_node);
            let sinks = fanouts
                .iter()
                .filter(move |(_, sink_pin)| commutative || *sink_pin == pin_idx)
                .map(|(sink, _)| *sink)
                .collect::<Vec<_>>();
            sinks
        })
        .filter(|v| !v.is_empty())
        .map(|v| v.into_iter().collect::<HashSet<CellRef<'d>>>());

    Some(intersect(sets))
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
    let Some(pattern_current) = pattern_mapping_queue.pop_front() else {
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
        pattern_current.debug_index(),
        pattern_index.get_node_type(pattern_current),
        pattern_mapping_queue.len(),
        node_mapping.len()
    );

    let current_type = pattern_index.get_node_type(pattern_current);

    // Base candidates (type-only)
    // let base_candidates = initial_candidates_for_type(design_index, current_type);

    // Constraint candidates
    let co_sinks =
        candidates_from_mapped_sinks(pattern_current, pattern_index, design_index, &node_mapping);
    let co_sources = candidates_from_mapped_sources(
        pattern_current,
        current_type,
        pattern_index,
        design_index,
        &node_mapping,
    );

    // If this is an Input with no constraints yet and there are still other nodes to map,
    // defer it by pushing it to the back and move on. This prevents wildcard blow-up.
    if matches!(current_type, NodeType::Input)
        && co_sinks.is_none()
        && co_sources.is_none()
        && !pattern_mapping_queue.is_empty()
    {
        pattern_mapping_queue.push_back(pattern_current);
        return find_isomorphisms_recursive(
            pattern_index,
            design_index,
            config,
            node_mapping,
            pattern_mapping_queue,
            input_by_name,
            output_by_name,
            depth,
        );
    }

    let mut constraints: Vec<HashSet<CellRef<'d>>> = Vec::new();

    match co_sinks {
        Some(sinks) => {
            tracing::event!(
                tracing::Level::TRACE,
                "find_isomorphisms_recursive[depth={}]: co_sinks constraint size={}",
                depth,
                sinks.len()
            );
            constraints.push(sinks);
        }
        None => {}
    }

    match co_sources {
        Some(sources) => {
            tracing::event!(
                tracing::Level::TRACE,
                "find_isomorphisms_recursive[depth={}]: co_sources constraint size={}",
                depth,
                sources.len()
            );
            constraints.push(sources);
        }
        None => {}
    }

    if !matches!(current_type, NodeType::Input) {
        tracing::event!(
            tracing::Level::TRACE,
            "find_isomorphisms_recursive[depth={}]: pushing default for gate type",
            depth
        );
        constraints.push(initial_candidates_for_type(design_index, current_type));
    }

    if constraints.is_empty() {
        tracing::event!(
            tracing::Level::TRACE,
            "find_isomorphisms_recursive[depth={}]: no constraints found, pushing default for input",
            depth
        );
        constraints.push(initial_candidates_for_type(design_index, current_type));
    }

    // Candidate selection:
    // - For Input nodes: when constraints exist, start from the constraint set(s) only.
    //   Avoid building the "all design nodes" base.
    // - For non-Input nodes: start from the type-based base and intersect constraints.
    let candidates = intersect(constraints);

    // Function to be executed on each possible candidate
    let process_candidate = |d_node: CellRef<'d>| -> Vec<SubgraphIsomorphism<'p, 'd>> {
        if is_node_already_mapped(d_node, &node_mapping, depth) {
            return Vec::new();
        }
        if !are_nodes_compatible(current_type, design_index.get_node_type(d_node)) {
            return Vec::new();
        }
        if !is_node_connectivity_valid(
            pattern_current,
            d_node,
            pattern_index,
            design_index,
            config,
            &node_mapping,
        ) {
            return Vec::new();
        }

        let mut new_node_mapping = node_mapping.clone();
        new_node_mapping.insert(pattern_current, d_node);

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
