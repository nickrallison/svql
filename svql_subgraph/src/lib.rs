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
use std::iter::Once;
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

struct SubgraphRecurse<'a, 'p, 'd> {
    // Owned Values
    depth: usize,
    p_current: CellRef<'p>,
    node_mapping: NodeMapping<'p, 'd>,
    pattern_mapping_queue: VecDeque<CellRef<'p>>,

    // Reference to Top Caller Values
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    config: &'a Config,
    input_by_name: &'a HashMap<&'p str, CellRef<'p>>,
    output_by_name: &'a HashMap<&'p str, CellRef<'p>>,

    // Candidates iterator
    candidates: Candidates<'a, 'd>,

    // Constraints
    already_mapped_constraint: NotAlreadyMappedConstraint<'p, 'd>,
    connectivity_constraint: ConnectivityConstraint<'a, 'p, 'd>,
}

impl<'a, 'p, 'd> SubgraphRecurse<'a, 'p, 'd> {
    fn flat_map_candidates(self) -> Box<dyn Iterator<Item = SubgraphIsomorphism<'p, 'd>> + 'a> {
        let filtered_candidates = self
            .candidates
            .into_iter()
            .filter(|d_node| self.already_mapped_constraint.d_candidate_is_valid(d_node))
            .filter(|d_node| self.connectivity_constraint.d_candidate_is_valid(d_node));

        let results = filtered_candidates.flat_map(|d_candidate| {
            let mut new_node_mapping = self.node_mapping.clone();
            new_node_mapping.insert(self.p_current, d_candidate);

            // Recurse; extend the single result Vec (no per-branch Vec flattening).
            find_isomorphisms_recursive(
                self.pattern_index,
                self.design_index,
                self.config,
                new_node_mapping,
                self.pattern_mapping_queue.clone(),
                self.input_by_name,
                self.output_by_name,
                self.depth + 1,
            )
        });
        Box::new(results)
    }
}

enum SubgraphRecurseEnum<'a, 'p, 'd> {
    Rec(SubgraphRecurse<'a, 'p, 'd>),
    Base(SubgraphIsomorphism<'p, 'd>),
}

impl<'a, 'p, 'd> SubgraphRecurseEnum<'a, 'p, 'd> {
    fn flat_map_candidates(self) -> Box<dyn Iterator<Item = SubgraphIsomorphism<'p, 'd>> + 'a> {
        match self {
            SubgraphRecurseEnum::Rec(rec) => rec.flat_map_candidates(),
            SubgraphRecurseEnum::Base(base) => {
                let iter = std::iter::once(base);
                Box::new(iter)
            }
        }
    }
}

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

    let mut results: Vec<SubgraphIsomorphism> = find_isomorphisms_recursive(
        &pattern_index,
        &design_index,
        config,
        initial_node_mapping,
        pattern_mapping_queue,
        &input_by_name,
        &output_by_name,
        0, // depth
    )
    .collect();

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

fn find_isomorphisms_recursive<'a, 'p, 'd>(
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    config: &'a Config,
    node_mapping: NodeMapping<'p, 'd>,
    mut pattern_mapping_queue: VecDeque<CellRef<'p>>,
    input_by_name: &'a HashMap<&'p str, CellRef<'p>>,
    output_by_name: &'a HashMap<&'p str, CellRef<'p>>,
    depth: usize,
) -> SubgraphRecurseEnum<'a, 'p, 'd> {
    let Some(pattern_current) = pattern_mapping_queue.pop_front() else {
        return SubgraphRecurseEnum::Base(std::iter::once(SubgraphIsomorphism {
            mapping: node_mapping,
            input_by_name: input_by_name.clone(),
            output_by_name: output_by_name.clone(),
        }));
    };

    let current_type = NodeType::from(pattern_current.get().as_ref());

    let node_constraints = {
        let design_sinks_constraints =
            DesignSinkConstraint::new(pattern_current, pattern_index, design_index, &node_mapping)
                .get_candidates_owned();
        let sources_constraints = DesignSourceConstraint::new(
            pattern_current,
            pattern_index,
            design_index,
            &node_mapping,
        )
        .get_candidates_owned();
        design_sinks_constraints.intersect(sources_constraints)
    };
    let already_mapped_constraint = NotAlreadyMappedConstraint::new(node_mapping.clone());
    let connectivity_constraint = ConnectivityConstraint::new(
        pattern_current,
        pattern_index,
        design_index,
        config,
        node_mapping.clone(),
    );

    // let candidates = match node_constraints.get_candidates_owned() {
    //     Some(set) => Either::Left(set.into_iter()),
    //     None => Either::Right(initial_candidates(design_index, current_type).copied()),
    // }
    // .filter(|d_node| already_mapped_constraint.d_candidate_is_valid(d_node))
    // .filter(|d_node| connectivity_constraint.d_candidate_is_valid(d_node));

    let candidates: Box<dyn Iterator<Item = CellRef<'d>>> =
        match node_constraints.get_candidates_owned() {
            Some(set) => Box::new(set.into_iter()),
            None => Box::new(initial_candidates(design_index, current_type).copied()),
        };

    let recurse_iter: SubgraphRecurse<'a, 'p, 'd> = SubgraphRecurse {
        // Owned
        depth,
        p_current: pattern_current,
        node_mapping,
        pattern_mapping_queue,

        // References
        pattern_index,
        design_index,
        config,
        input_by_name,
        output_by_name,

        // Candidates Iter
        candidates,
        already_mapped_constraint,
        connectivity_constraint,
    };

    SubgraphRecurseEnum::Rec(recurse_iter)

    // Accumulate results locally (no &mut parameters). This keeps a simple path
    // to change candidates_it.into_par_iter() and reduce results with Rayon later.

    // let results = candidates.flat_map(|d_candidate| {
    //     let mut new_node_mapping = node_mapping.clone();
    //     new_node_mapping.insert(pattern_current, d_candidate);

    //     // Recurse; extend the single result Vec (no per-branch Vec flattening).
    //     find_isomorphisms_recursive(
    //         pattern_index,
    //         design_index,
    //         config,
    //         new_node_mapping,
    //         pattern_mapping_queue.clone(),
    //         input_by_name,
    //         output_by_name,
    //         depth + 1,
    //     )
    // });

    // Either::Right(results)
}
