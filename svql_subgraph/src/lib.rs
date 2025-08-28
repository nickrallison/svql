#![allow(dead_code)]
mod constraints;
mod graph_index;
mod isomorphism;
mod node;

use graph_index::GraphIndex;
use isomorphism::NodeMapping;

use prjunnamed_netlist::{CellRef, Design};

use crate::constraints::{
    ConnectivityConstraint, Constraint, NodeConstraints, NotAlreadyMappedConstraint,
};
use crate::node::{NodeSource, NodeType};
use itertools::Either;
use std::collections::{HashMap, HashSet, VecDeque};
use svql_common::{Config, DedupeMode};

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

    let mut results = find_isomorphisms_recursive(
        &pattern_index,
        &design_index,
        config,
        initial_node_mapping,
        pattern_mapping_queue,
        &input_by_name,
        &output_by_name,
        0, // depth
    );

    // Dedupe ONCE at the top-level; avoid per-depth overhead in the hot path.
    if matches!(config.dedupe, DedupeMode::AutoMorph) {
        let mut seen: HashSet<Vec<usize>> = HashSet::new();
        results.retain(|m| seen.insert(m.mapping.signature()));
    }

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
fn initial_candidates<'d, 'a>(
    design_index: &'a GraphIndex<'d>,
    current_type: NodeType,
) -> impl Iterator<Item = &'a CellRef<'d>> {
    match current_type {
        NodeType::Input => design_index.get_nodes_topo().iter(),
        _ => design_index.get_by_type(current_type).iter(),
    }
}

/// Candidates restricted by already-mapped sinks (fan-out constraints).
/// For each mapped sink of the pattern node, compute the set of possible drivers
/// in the design and intersect across sinks. If none of the sinks are mapped,
/// returns None (no restriction).
fn design_sinks_constraints<'p, 'd>(
    pattern_current: CellRef<'p>,
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> NodeConstraints<'d> {
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
        return NodeConstraints::new(None);
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
        .map(|v| v.into_iter().collect::<HashSet<CellRef<'d>>>())
        .map(|s| NodeConstraints::new(Some(s)));

    NodeConstraints::intersect_many(sets)
}

/// Candidates restricted by already-mapped sources (fan-in constraints).
/// For each mapped source of the pattern node, collect the sinks in the design
/// that are driven by the corresponding mapped design driver, respecting
/// commutativity of the current node. Intersect across all mapped sources.
/// If no sources are mapped, returns None (no restriction).
fn design_sources_constraints<'p, 'd>(
    pattern_current: CellRef<'p>,
    current_type: NodeType,
    pattern_index: &GraphIndex<'p>,
    design_index: &GraphIndex<'d>,
    mapping: &NodeMapping<'p, 'd>,
) -> NodeConstraints<'d> {
    let commutative = current_type.has_commutative_inputs();

    let mapped_sources: Vec<(usize, NodeSource<'p>)> = pattern_index
        .get_node_sources(pattern_current)
        .iter()
        .cloned()
        .enumerate()
        .collect();

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
        .map(|v| v.into_iter().collect::<HashSet<CellRef<'d>>>())
        .map(|s| NodeConstraints::new(Some(s)));

    NodeConstraints::intersect_many(sets)

    // if sets.is_empty() {
    //     return None;
    // }

    // Some(intersect(sets))
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

    let base_candidates = initial_candidates(design_index, current_type);

    // Constraint candidates
    let sinks_constraints =
        design_sinks_constraints(pattern_current, pattern_index, design_index, &node_mapping);
    let sources_constraints = design_sources_constraints(
        pattern_current,
        current_type,
        pattern_index,
        design_index,
        &node_mapping,
    );
    let node_constraints =
        NodeConstraints::intersect_many(vec![sinks_constraints, sources_constraints]);
    let already_mapped_constraint = NotAlreadyMappedConstraint::new(&node_mapping);
    let connectivity_constraint = ConnectivityConstraint::new(
        pattern_current,
        pattern_index,
        design_index,
        config,
        &node_mapping,
    );

    let candidates = match node_constraints.get_candidates_owned() {
        Some(set) => Either::Left(set.into_iter()),
        None => Either::Right(base_candidates.copied()),
    }
    .filter(|d_node| already_mapped_constraint.d_candidate_is_valid(d_node))
    .filter(|d_node| connectivity_constraint.d_candidate_is_valid(d_node));

    // Accumulate results locally (no &mut parameters). This keeps a simple path
    // to change candidates_it.into_par_iter() and reduce results with Rayon later.
    let mut results: Vec<SubgraphIsomorphism<'p, 'd>> = candidates
        .flat_map(|d_candidate| {
            let mut new_node_mapping = node_mapping.clone();
            new_node_mapping.insert(pattern_current, d_candidate);

            // Recurse; extend the single result Vec (no per-branch Vec flattening).
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
        })
        .collect();

    results
}
