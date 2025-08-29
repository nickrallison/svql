#![allow(dead_code)]
mod candidates;
mod constraints;
mod graph_index;
mod isomorphism;
mod node;

pub mod profiling;
pub mod progress;

use graph_index::GraphIndex;
use isomorphism::NodeMapping;

use crate::candidates::{BaseIter, FilteredCandidates};
use crate::constraints::{
    ConnectivityConstraint, Constraint, DesignSinkConstraint, DesignSourceConstraint,
    NotAlreadyMappedConstraint,
};
use crate::node::NodeType;
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
        None
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
    let pattern_index = GraphIndex::build(pattern);
    let design_index = GraphIndex::build(design);

    if pattern_index.gate_count() == 0 || design_index.gate_count() == 0 {
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

    // Estimate total candidates up‑front: sum initial candidate counts per pattern node.
    let total_candidates = estimate_total_candidates(&pattern_mapping_queue, &design_index);
    progress.set_total_candidates(total_candidates as u64);

    let initial_node_mapping: NodeMapping<'p, 'd> = NodeMapping::new();

    let mut results = find_isomorphisms_recursive_collect(
        &pattern_index,
        &design_index,
        config,
        initial_node_mapping,
        pattern_mapping_queue,
        &input_by_name,
        &output_by_name,
        0, // depth
        Some(progress),
    );

    if matches!(config.dedupe, DedupeMode::AutoMorph) {
        let mut seen: HashSet<Vec<usize>> = HashSet::new();
        results.retain(|m| seen.insert(m.mapping.signature()));
    }

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
    config: &'a Config,
    node_mapping: &NodeMapping<'p, 'd>,
    progress: Option<&'g progress::Progress>,
) -> FilteredCandidates<'a, 'p, 'd, 'g> {
    let current_type = NodeType::from(pattern_current.get().as_ref());

    let node_constraints = {
        let design_sinks_constraints =
            DesignSinkConstraint::new(pattern_current, pattern_index, design_index, node_mapping)
                .get_candidates_owned();
        let sources_constraints =
            DesignSourceConstraint::new(pattern_current, pattern_index, design_index, node_mapping)
                .get_candidates_owned();
        design_sinks_constraints.intersect(sources_constraints)
    };

    let base = match node_constraints.get_candidates_owned() {
        Some(set) => BaseIter::Constrained(set.into_iter()),
        None => initial_candidates(design_index, current_type),
    };

    let already_mapped_constraint = NotAlreadyMappedConstraint::new(node_mapping.clone());
    let connectivity_constraint = ConnectivityConstraint::new(
        pattern_current,
        pattern_index,
        design_index,
        config,
        node_mapping.clone(),
    );

    FilteredCandidates {
        base,
        already_mapped: already_mapped_constraint,
        connectivity: connectivity_constraint,
        progress,
    }
}

fn find_isomorphisms_recursive_collect<'a, 'p, 'd>(
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    config: &'a Config,
    node_mapping: NodeMapping<'p, 'd>,
    mut pattern_mapping_queue: VecDeque<CellRef<'p>>,
    input_by_name: &'a HashMap<&'p str, CellRef<'p>>,
    output_by_name: &'a HashMap<&'p str, CellRef<'p>>,
    depth: usize,
    progress: Option<&'a progress::Progress>,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    let Some(pattern_current) = pattern_mapping_queue.pop_front() else {
        return vec![SubgraphIsomorphism {
            mapping: node_mapping,
            input_by_name: input_by_name.clone(),
            output_by_name: output_by_name.clone(),
        }];
    };

    let candidates_iter = build_filtered_candidates(
        pattern_current,
        pattern_index,
        design_index,
        config,
        &node_mapping,
        progress,
    );

    #[cfg(feature = "rayon")]
    let cand_iter = candidates_iter.collect::<Vec<_>>().into_par_iter();

    #[cfg(not(feature = "rayon"))]
    let cand_iter = candidates_iter.collect::<Vec<_>>().into_iter();

    cand_iter
        .flat_map(|d_candidate| {
            let mut nm = node_mapping.clone();
            nm.insert(pattern_current, d_candidate);
            find_isomorphisms_recursive_collect(
                pattern_index,
                design_index,
                config,
                nm,
                pattern_mapping_queue.clone(),
                input_by_name,
                output_by_name,
                depth + 1,
                progress,
            )
        })
        .collect()
}
