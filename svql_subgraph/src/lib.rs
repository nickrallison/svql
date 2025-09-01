#![allow(dead_code)]
mod candidates;
mod constraints;
mod graph_index;
mod isomorphism;

pub(crate) mod node;

pub mod profiling;
pub mod progress;

use graph_index::GraphIndex;
use isomorphism::NodeMapping;
use tracing::{debug, info, trace};

use crate::candidates::{BaseIter, FilteredCandidates};
use crate::constraints::{
    ConnectivityConstraint, Constraint, DesignSinkConstraint, DesignSourceConstraint,
    NotAlreadyMappedConstraint,
};
use crate::node::{NodeSource, NodeType};
use prjunnamed_netlist::{CellRef, Design};

use std::collections::{HashMap, HashSet, VecDeque};
use svql_common::{Config, DedupeMode};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct SubgraphIsomorphism<'p, 'd> {
    // Mapping of pattern nodes to design nodes (and reverse)
    mapping: NodeMapping<'p, 'd>,

    // Boundary IO lookup tables
    pub input_by_name: HashMap<&'p str, CellRef<'p>>,
    pub output_by_name: HashMap<&'p str, CellRef<'p>>,

    // For each named output in the pattern, the driver cell/bit in the design for each bit
    // index of that output (usually single-bit in our patterns). Indexed by output name, then bit.
    output_driver_by_name: HashMap<&'p str, Vec<(CellRef<'d>, usize)>>,
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
        name: &str,
        bit: usize,
    ) -> Option<(CellRef<'d>, usize)> {
        self.output_driver_by_name
            .get(name)
            .and_then(|v| v.get(bit).copied())
    }
}

/// Backward‑compatible API (no progress handle returned).
pub fn find_subgraph_isomorphisms<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    let progress = progress::Progress::new();
    find_subgraph_isomorphisms_with_progress(pattern, design, config, &progress)
}

/// New API that updates the provided `progress` atomically as the search proceeds.
pub fn find_subgraph_isomorphisms_with_progress<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
    progress: &progress::Progress,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    info!("Starting subgraph isomorphism search");
    trace!("Config: {:?}", config);

    let pattern_index = GraphIndex::build(pattern);
    let design_index = GraphIndex::build(design);

    info!(
        "Built indices: pattern has {} nodes, design has {} nodes",
        pattern_index.gate_count(),
        design_index.gate_count()
    );

    if pattern_index.gate_count() == 0 || design_index.gate_count() == 0 {
        info!("Empty pattern or design, returning empty results");
        return Vec::new();
    }

    // Build boundary IO maps for the pattern (by name).
    let input_by_name: HashMap<&'p str, CellRef<'p>> = pattern_index
        .get_nodes_topo()
        .iter()
        .filter(|c| matches!(NodeType::from(c.get().as_ref()), NodeType::Input))
        .filter_map(|c| pattern_index.get_input_name(*c).map(|n| (n, *c)))
        .collect();

    let output_by_name: HashMap<&'p str, CellRef<'p>> = pattern_index
        .get_nodes_topo()
        .iter()
        .filter(|c| matches!(NodeType::from(c.get().as_ref()), NodeType::Output))
        .filter_map(|c| pattern_index.get_output_name(*c).map(|n| (n, *c)))
        .collect();

    debug!(
        "Pattern inputs: {:?}",
        input_by_name.keys().collect::<Vec<_>>()
    );
    debug!(
        "Pattern outputs: {:?}",
        output_by_name.keys().collect::<Vec<_>>()
    );

    // in topological_order, only gates & inputs (push inputs to the back)
    let pattern_mapping_queue: VecDeque<CellRef<'p>> = {
        let mut q: Vec<CellRef<'p>> = pattern_index
            .get_nodes_topo()
            .iter()
            .filter(|c| !matches!(NodeType::from(c.get().as_ref()), NodeType::Output))
            .copied()
            .collect();

        q.sort_by(|a, b| {
            let a_is_input = matches!(NodeType::from(a.get().as_ref()), NodeType::Input);
            let b_is_input = matches!(NodeType::from(b.get().as_ref()), NodeType::Input);
            a_is_input.cmp(&b_is_input)
        });

        q.into()
    };

    debug!(
        "Pattern mapping queue size: {}",
        pattern_mapping_queue.len()
    );

    // Estimate total candidates up‑front: sum initial candidate counts per pattern node.
    let total_candidates = estimate_total_candidates(&pattern_mapping_queue, &design_index);
    progress.set_total_candidates(total_candidates as u64);
    info!("Estimated total candidates: {}", total_candidates);

    let initial_node_mapping: NodeMapping<'p, 'd> = NodeMapping::new();

    let mut results = find_isomorphisms_recursive_collect(
        &pattern_index,
        &design_index,
        pattern,
        design,
        config,
        initial_node_mapping,
        pattern_mapping_queue,
        &input_by_name,
        &output_by_name,
        0, // depth
        Some(progress),
    );

    info!(
        "Found {} initial results before deduplication",
        results.len()
    );

    if matches!(config.dedupe, DedupeMode::AutoMorph) {
        let mut seen: HashSet<Vec<usize>> = HashSet::new();
        results.retain(|m| seen.insert(m.mapping.signature()));
        info!("After AutoMorph deduplication: {} results", results.len());
    }

    info!("Final result count: {}", results.len());
    results
}

fn estimate_total_candidates<'p, 'd>(
    pattern_queue: &VecDeque<CellRef<'p>>,
    design_index: &GraphIndex<'d>,
) -> usize {
    pattern_queue
        .iter()
        .map(|p_node| {
            let ty = NodeType::from(p_node.get().as_ref());
            match ty {
                NodeType::Input => design_index.get_nodes_topo().len(),
                _ => design_index.get_by_type(ty).len(),
            }
        })
        .sum()
}

fn initial_candidates<'d, 'a>(
    design_index: &'a GraphIndex<'d>,
    current_type: NodeType,
) -> BaseIter<'a, 'd> {
    let slice: &'a [CellRef<'d>] = match current_type {
        NodeType::Input => design_index.get_nodes_topo(),
        _ => design_index.get_by_type(current_type),
    };
    BaseIter::Unconstrained(slice.iter())
}

fn build_filtered_candidates<'a, 'p, 'd, 'g>(
    pattern_current: CellRef<'p>,
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    pattern: &'p Design,
    design: &'d Design,
    config: &'a Config,
    node_mapping: &NodeMapping<'p, 'd>,
    progress: Option<&'g progress::Progress>,
) -> FilteredCandidates<'a, 'p, 'd, 'g> {
    let current_type = NodeType::from(pattern_current.get().as_ref());
    trace!(
        "Building filtered candidates for pattern node {:?} of type {:?}",
        pattern_current, current_type
    );

    let node_constraints = {
        trace!("Computing design sink constraints");
        let design_sinks_constraints =
            DesignSinkConstraint::new(pattern_current, pattern_index, design_index, node_mapping)
                .get_candidates_owned();
        trace!("Computing design source constraints");
        let sources_constraints =
            DesignSourceConstraint::new(pattern_current, pattern_index, design_index, node_mapping)
                .get_candidates_owned();
        let intersection = design_sinks_constraints.intersect(sources_constraints);
        let candidate_count = intersection
            .get_candidates()
            .map_or("unlimited".to_string(), |s| s.len().to_string());
        trace!(
            "Node constraints intersection has {} candidates",
            candidate_count
        );
        intersection
    };

    let base = match node_constraints.get_candidates_owned() {
        Some(set) => {
            trace!(
                "Using constrained base iterator with {} candidates",
                set.len()
            );
            BaseIter::Constrained(set.into_iter())
        }
        None => {
            let candidates = initial_candidates(design_index, current_type);
            trace!(
                "Using unconstrained base iterator for type {:?}",
                current_type
            );
            candidates
        }
    };

    let already_mapped_constraint = NotAlreadyMappedConstraint::new(node_mapping.clone());
    let connectivity_constraint = ConnectivityConstraint::new(
        pattern_current,
        pattern_index,
        design_index,
        pattern,
        design,
        config,
        node_mapping.clone(),
    );

    trace!("Created filtered candidates iterator");
    FilteredCandidates {
        base,
        already_mapped: already_mapped_constraint,
        connectivity: connectivity_constraint,
        progress,
    }
}

fn compute_output_drivers<'p, 'd>(
    pattern_index: &GraphIndex<'p>,
    _design_index: &GraphIndex<'d>,
    output_by_name: &HashMap<&'p str, CellRef<'p>>,
    mapping: &NodeMapping<'p, 'd>,
) -> HashMap<&'p str, Vec<(CellRef<'d>, usize)>> {
    let mut result: HashMap<&'p str, Vec<(CellRef<'d>, usize)>> = HashMap::new();

    for (&name, &p_out) in output_by_name.iter() {
        // For each bit of the pattern output, find its source in the pattern,
        // map that source node to the design node, and record (design_node, bit).
        let sources = pattern_index.get_node_sources(p_out);
        let mut vec_bits: Vec<(CellRef<'d>, usize)> = Vec::with_capacity(sources.len());

        for (_bit_idx, src) in sources.iter().enumerate() {
            match src {
                NodeSource::Gate(p_src_node, p_bit) | NodeSource::Io(p_src_node, p_bit) => {
                    if let Some(d_src_node) = mapping.get_design_node(*p_src_node) {
                        vec_bits.push((d_src_node, *p_bit));
                    } else {
                        // Unmapped source — should not happen for complete mapping; skip.
                    }
                }
                NodeSource::Const(_t) => {
                    // Outputs driven by consts are not used by current patterns; skip.
                }
            }
        }

        // If there are no sources (shouldn't happen for normal outputs), leave empty.
        result.insert(name, vec_bits);
    }

    result
}

fn find_isomorphisms_recursive_collect<'a, 'p, 'd>(
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    pattern: &'p Design,
    design: &'d Design,
    config: &'a Config,
    node_mapping: NodeMapping<'p, 'd>,
    mut pattern_mapping_queue: VecDeque<CellRef<'p>>,
    input_by_name: &'a HashMap<&'p str, CellRef<'p>>,
    output_by_name: &'a HashMap<&'p str, CellRef<'p>>,
    depth: usize,
    progress: Option<&'a progress::Progress>,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    trace!(
        "Recursive call at depth {}, queue size: {}, current mapping size: {}",
        depth,
        pattern_mapping_queue.len(),
        node_mapping.len()
    );

    let Some(pattern_current) = pattern_mapping_queue.pop_front() else {
        // Complete assignment: compute output drivers for boundary lookup
        debug!("Complete assignment reached at depth {}", depth);
        let output_driver_by_name =
            compute_output_drivers(pattern_index, design_index, output_by_name, &node_mapping);

        trace!(
            "Created complete isomorphism with {} mappings",
            node_mapping.len()
        );
        return vec![SubgraphIsomorphism {
            mapping: node_mapping,
            input_by_name: input_by_name.clone(),
            output_by_name: output_by_name.clone(),
            output_driver_by_name,
        }];
    };

    let pattern_current_type = NodeType::from(pattern_current.get().as_ref());
    debug!(
        "Processing pattern node {:?} of type {:?} at depth {}",
        pattern_current, pattern_current_type, depth
    );

    let candidates_iter = build_filtered_candidates(
        pattern_current,
        pattern_index,
        design_index,
        pattern,
        design,
        config,
        &node_mapping,
        progress,
    );

    let candidates_vec: Vec<_> = candidates_iter.collect();
    debug!(
        "Found {} candidates for pattern node {:?}",
        candidates_vec.len(),
        pattern_current
    );

    if candidates_vec.is_empty() {
        debug!(
            "No valid candidates found for pattern node {:?}, backtracking",
            pattern_current
        );
        return Vec::new();
    }

    #[cfg(feature = "rayon")]
    let cand_iter = candidates_vec.into_par_iter();

    #[cfg(not(feature = "rayon"))]
    let cand_iter = candidates_vec.into_iter();

    let results: Vec<SubgraphIsomorphism<'p, 'd>> = cand_iter
        .flat_map(|d_candidate| {
            trace!(
                "Trying candidate {:?} for pattern node {:?}",
                d_candidate, pattern_current
            );
            let mut nm = node_mapping.clone();
            nm.insert(pattern_current, d_candidate);
            find_isomorphisms_recursive_collect(
                pattern_index,
                design_index,
                pattern,
                design,
                config,
                nm,
                pattern_mapping_queue.clone(),
                input_by_name,
                output_by_name,
                depth + 1,
                progress,
            )
        })
        .collect();

    debug!("Depth {} returning {} results", depth, results.len());
    results
}
