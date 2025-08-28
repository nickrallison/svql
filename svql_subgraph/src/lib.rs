#![allow(dead_code)]
mod candidates;
mod constraints;
mod graph_index;
mod isomorphism;
mod node;

use graph_index::GraphIndex;
use isomorphism::NodeMapping;

use prjunnamed_netlist::{CellRef, Design};

use crate::candidates::Candidates;
use crate::constraints::{
    ConnectivityConstraint, Constraint, DesignSinkConstraint, DesignSourceConstraint,
    NotAlreadyMappedConstraint,
};
use crate::node::NodeType;

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

/// An explicit, stack-based DFS iterator over all subgraph isomorphisms.
/// It yields owned SubgraphIsomorphism values lazily.
struct SubgraphSearch<'a, 'p, 'd> {
    // Shared, read-only context
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    config: &'a Config,

    input_by_name: HashMap<&'p str, CellRef<'p>>,
    output_by_name: HashMap<&'p str, CellRef<'p>>,

    // DFS stack
    stack: Vec<Frame<'p, 'd>>,
}

struct Frame<'p, 'd> {
    p_current: CellRef<'p>,
    mapping: NodeMapping<'p, 'd>,
    queue: VecDeque<CellRef<'p>>,
    candidates: Vec<CellRef<'d>>,
    next_idx: usize,
}

impl<'a, 'p, 'd> SubgraphSearch<'a, 'p, 'd> {
    fn new(
        pattern_index: &'a GraphIndex<'p>,
        design_index: &'a GraphIndex<'d>,
        config: &'a Config,
        input_by_name: HashMap<&'p str, CellRef<'p>>,
        output_by_name: HashMap<&'p str, CellRef<'p>>,
        initial_queue: VecDeque<CellRef<'p>>,
        initial_mapping: NodeMapping<'p, 'd>,
    ) -> Self {
        let mut this = Self {
            pattern_index,
            design_index,
            config,
            input_by_name,
            output_by_name,
            stack: Vec::new(),
        };

        if let Some(frame) = this.make_frame(initial_mapping, initial_queue) {
            this.stack.push(frame);
        }

        this
    }

    fn make_frame(
        &self,
        mapping: NodeMapping<'p, 'd>,
        mut queue: VecDeque<CellRef<'p>>,
    ) -> Option<Frame<'p, 'd>> {
        let p_current = queue.pop_front()?;
        let current_type = NodeType::from(p_current.get().as_ref());

        // Build node candidate constraints (source/sink) based on already-mapped neighbors.
        let node_constraints = {
            let design_sinks_constraints = DesignSinkConstraint::new(
                p_current,
                self.pattern_index,
                self.design_index,
                &mapping,
            )
            .get_candidates_owned();
            let sources_constraints = DesignSourceConstraint::new(
                p_current,
                self.pattern_index,
                self.design_index,
                &mapping,
            )
            .get_candidates_owned();
            design_sinks_constraints.intersect(sources_constraints)
        };

        let already_mapped_constraint = NotAlreadyMappedConstraint::new(mapping.clone());
        let connectivity_constraint = ConnectivityConstraint::new(
            p_current,
            self.pattern_index,
            self.design_index,
            self.config,
            mapping.clone(),
        );

        let base_candidates: Candidates<'_, 'd> = match node_constraints.get_candidates_owned() {
            Some(set) => Candidates::Constrained(set),
            None => initial_candidates(self.design_index, current_type),
        };

        // Fully filter candidates now; store as a Vec in the frame
        let candidates = base_candidates
            .iter()
            .filter(|d_node| already_mapped_constraint.d_candidate_is_valid(d_node))
            .filter(|d_node| connectivity_constraint.d_candidate_is_valid(d_node))
            .collect::<Vec<_>>();

        Some(Frame {
            p_current,
            mapping,
            queue,
            candidates,
            next_idx: 0,
        })
    }

    fn finish_mapping(&self, mapping: NodeMapping<'p, 'd>) -> SubgraphIsomorphism<'p, 'd> {
        SubgraphIsomorphism {
            mapping,
            input_by_name: self.input_by_name.clone(),
            output_by_name: self.output_by_name.clone(),
        }
    }
}

impl<'a, 'p, 'd> Iterator for SubgraphSearch<'a, 'p, 'd> {
    type Item = SubgraphIsomorphism<'p, 'd>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(top) = self.stack.last_mut() else {
                return None;
            };

            if top.next_idx >= top.candidates.len() {
                // Exhausted this frame; backtrack.
                self.stack.pop();
                continue;
            }

            let d_candidate = top.candidates[top.next_idx];
            top.next_idx += 1;

            // Create new mapping by assigning p_current -> d_candidate
            let mut new_mapping = top.mapping.clone();
            new_mapping.insert(top.p_current, d_candidate);

            // Advance the queue for the next frame
            let next_queue = top.queue.clone();

            // If there is no more pattern node to map, we have a full isomorphism.
            if next_queue.is_empty() {
                return Some(self.finish_mapping(new_mapping));
            }

            // Otherwise, descend by creating a new frame for the next pattern node.
            if let Some(frame) = self.make_frame(new_mapping, next_queue) {
                self.stack.push(frame);
            }
        }
    }
}

// impl<'a, 'p, 'd> IntoIterator for SubgraphSearch<'a, 'p, 'd> {
//     type Item = SubgraphIsomorphism<'p, 'd>;
//     type IntoIter = Self;

//     fn into_iter(self) -> Self::IntoIter {
//         self
//     }
// }

pub fn find_subgraph_isomorphisms<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    let pattern_index = GraphIndex::build(pattern);
    let design_index = GraphIndex::build(design);

    if pattern_index.node_count() == 0 || design_index.node_count() == 0 {
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

    let pattern_mapping_queue: VecDeque<CellRef<'p>> = {
        // in topological_order, only gates & inputs
        let mut initial_pattern_mapping: Vec<CellRef<'p>> = pattern_index
            .get_nodes_topo()
            .iter()
            .filter(|c| !matches!(NodeType::from(c.get().as_ref()), NodeType::Output))
            .copied()
            .collect();

        // stable sort inputs to back
        initial_pattern_mapping.sort_by(|a, b| {
            let a_is_input = matches!(NodeType::from(a.get().as_ref()), NodeType::Input);
            let b_is_input = matches!(NodeType::from(b.get().as_ref()), NodeType::Input);
            a_is_input.cmp(&b_is_input)
        });

        initial_pattern_mapping.into()
    };

    let initial_node_mapping: NodeMapping<'p, 'd> = NodeMapping::new();

    // Lazy, stack-based DFS iterator
    let search = SubgraphSearch::new(
        &pattern_index,
        &design_index,
        config,
        input_by_name,
        output_by_name,
        pattern_mapping_queue,
        initial_node_mapping,
    );

    let mut results: Vec<SubgraphIsomorphism> = search.collect();

    // Dedupe ONCE at the top-level; avoid per-depth overhead in the hot path.
    if matches!(config.dedupe, DedupeMode::AutoMorph) {
        let mut seen: HashSet<Vec<usize>> = HashSet::new();
        results.retain(|m| seen.insert(m.mapping.signature()));
    }

    results
}

fn initial_candidates<'d, 'a>(
    design_index: &'a GraphIndex<'d>,
    current_type: NodeType,
) -> Candidates<'a, 'd> {
    let slice: &'a [CellRef<'d>] = match current_type {
        NodeType::Input => design_index.get_nodes_topo(),
        _ => design_index.get_by_type(current_type),
    };
    Candidates::Unconstrained(slice)
}
