//! Core subgraph isomorphism matching algorithm.

use std::collections::{HashSet, VecDeque};

use prjunnamed_netlist::Design;
use svql_common::Config;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::cell::{CellKind, CellWrapper};
use crate::embedding::{Embedding, EmbeddingSet};
use crate::graph_index::GraphIndex;
use crate::mapping::Assignment;
use crate::utils::intersect_sets;

/// Entry point for executing subgraph isomorphism searches.
pub struct SubgraphMatcher<'needle, 'haystack, 'cfg> {
    needle: &'needle Design,
    haystack: &'haystack Design,
    needle_index: GraphIndex<'needle>,
    haystack_index: GraphIndex<'haystack>,
    pub(crate) config: &'cfg Config,
}

/// Internal state for the backtracking search algorithm.
pub struct SubgraphMatcherCore<'needle, 'haystack, 'cfg> {
    pub(crate) needle: &'needle Design,
    pub(crate) haystack: &'haystack Design,
    pub(crate) needle_index: &'cfg GraphIndex<'needle>,
    pub(crate) haystack_index: &'cfg GraphIndex<'haystack>,
    pub(crate) config: &'cfg Config,
}

impl<'needle, 'haystack, 'cfg> SubgraphMatcher<'needle, 'haystack, 'cfg> {
    /// Finds all embeddings of the needle design within the haystack design.
    pub fn enumerate_all(
        needle: &'needle Design,
        haystack: &'haystack Design,
        config: &'cfg Config,
    ) -> EmbeddingSet<'needle, 'haystack> {
        let needle_index = GraphIndex::build(needle);
        let haystack_index = GraphIndex::build(haystack);

        let matcher = SubgraphMatcherCore {
            needle,
            haystack,
            needle_index: &needle_index,
            haystack_index: &haystack_index,
            config,
        };

        matcher.enumerate_embeddings()
    }

    /// Finds all embeddings using pre-built graph indices.
    pub fn enumerate_with_indices(
        needle: &'needle Design,
        haystack: &'haystack Design,
        needle_index: &'cfg GraphIndex<'needle>,
        haystack_index: &'cfg GraphIndex<'haystack>,
        config: &'cfg Config,
    ) -> EmbeddingSet<'needle, 'haystack> {
        let matcher = SubgraphMatcherCore {
            needle,
            haystack,
            needle_index,
            haystack_index,
            config,
        };
        matcher.enumerate_embeddings()
    }
}

impl<'needle, 'haystack, 'cfg> SubgraphMatcherCore<'needle, 'haystack, 'cfg> {
    /// Orchestrates the backtracking search across internal gates and input ports.
    pub fn enumerate_embeddings(&self) -> EmbeddingSet<'needle, 'haystack> {
        let (input_queue, internal_queue) = self.prepare_search_queues();
        let initial_assignment = Assignment::new();

        let mut results =
            self.match_internal_cells(initial_assignment, internal_queue, input_queue, 0);

        self.apply_deduplication(&mut results);

        EmbeddingSet {
            items: results,
            needle_input_fanout_by_name: self.needle_index.get_input_fanout_by_name().clone(),
            needle_output_fanin_by_name: self.needle_index.get_output_fanin_by_name().clone(),
        }
    }

    /// Recursively matches internal logic gates using backtracking.
    fn match_internal_cells(
        &self,
        assignment: Assignment<'needle, 'haystack>,
        mut internal_queue: VecDeque<CellWrapper<'needle>>,
        input_queue: VecDeque<CellWrapper<'needle>>,
        depth: usize,
    ) -> Vec<Embedding<'needle, 'haystack>> {
        let Some(current_needle) = internal_queue.pop_front() else {
            return self.match_input_cells(assignment, input_queue, depth + 1);
        };

        let candidates = self.find_candidates_for_cell(current_needle.clone(), &assignment);

        #[cfg(feature = "rayon")]
        let iter = candidates.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = candidates.into_iter();

        iter.flat_map(|candidate| {
            let mut next_assignment = assignment.clone();
            next_assignment.assign(current_needle.clone(), candidate);

            self.match_internal_cells(
                next_assignment,
                internal_queue.clone(),
                input_queue.clone(),
                depth + 1,
            )
        })
        .collect()
    }

    /// Recursively matches input ports after internal logic is assigned.
    fn match_input_cells(
        &self,
        assignment: Assignment<'needle, 'haystack>,
        mut input_queue: VecDeque<CellWrapper<'needle>>,
        depth: usize,
    ) -> Vec<Embedding<'needle, 'haystack>> {
        let Some(current_needle) = input_queue.pop_front() else {
            return vec![Embedding { assignment }];
        };

        let candidates = self.find_candidates_for_input(current_needle.clone(), &assignment);

        if candidates.is_empty() && self.config.pattern_vars_match_design_consts {
            return self.match_input_cells(assignment, input_queue, depth + 1);
        }

        #[cfg(feature = "rayon")]
        let iter = candidates.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = candidates.into_iter();

        iter.flat_map(|candidate| {
            let mut next_assignment = assignment.clone();
            next_assignment.assign(current_needle.clone(), candidate);

            self.match_input_cells(next_assignment, input_queue.clone(), depth + 1)
        })
        .collect()
    }

    /// Prepares work queues by ordering cells topologically.
    fn prepare_search_queues(
        &self,
    ) -> (
        VecDeque<CellWrapper<'needle>>,
        VecDeque<CellWrapper<'needle>>,
    ) {
        let mut inputs = VecDeque::new();
        let mut gates = VecDeque::new();

        for cell in self.needle_index.cells_topo().iter().rev() {
            match cell.cell_type() {
                CellKind::Output => continue,
                CellKind::Input => inputs.push_back(cell.clone()),
                _ => gates.push_back(cell.clone()),
            }
        }

        (inputs, gates)
    }

    /// Filters haystack cells that satisfy connectivity constraints for a needle cell.
    fn find_candidates_for_cell(
        &self,
        needle_cell: CellWrapper<'needle>,
        assignment: &Assignment<'needle, 'haystack>,
    ) -> Vec<CellWrapper<'haystack>> {
        let kind = needle_cell.cell_type();
        let needle_fanin = self
            .needle_index
            .fanin_with_ports(&needle_cell)
            .unwrap_or_default();

        let mapped_haystack_fanin: Vec<_> = needle_fanin
            .iter()
            .filter_map(|(p, _)| assignment.get_haystack_cell(p.clone()))
            .collect();

        let unfiltered: Vec<CellWrapper<'haystack>> = if mapped_haystack_fanin.is_empty() {
            self.haystack_index
                .cells_of_type_iter(kind)
                .map(|i| i.cloned().collect())
                .unwrap_or_default()
        } else {
            let fanout_sets: Vec<_> = mapped_haystack_fanin
                .iter()
                .filter_map(|c| self.haystack_index.fanout_set(c))
                .collect();
            intersect_sets(fanout_sets).into_iter().collect()
        };

        unfiltered
            .into_iter()
            .filter(|h| h.cell_type() == kind)
            .filter(|h| assignment.get_needle_cell(h.clone()).is_none())
            .filter(|h| self.check_fanin_constraints(needle_cell.clone(), h.clone(), assignment))
            .collect()
    }

    /// Filters haystack cells that can map to a needle input port.
    fn find_candidates_for_input(
        &self,
        needle_input: CellWrapper<'needle>,
        assignment: &Assignment<'needle, 'haystack>,
    ) -> Vec<CellWrapper<'haystack>> {
        let needle_fanout = self
            .needle_index
            .fanout_with_ports(&needle_input)
            .unwrap_or_default();

        let mapped_haystack_fanout: Vec<_> = needle_fanout
            .iter()
            .filter_map(|(p, _)| assignment.get_haystack_cell(p.clone()))
            .collect();

        let fanin_sets: Vec<_> = mapped_haystack_fanout
            .iter()
            .filter_map(|c| self.haystack_index.fanin_set(c))
            .collect();

        intersect_sets(fanin_sets)
            .into_iter()
            .filter(|h| {
                let mut next_assignment = assignment.clone();
                next_assignment.assign(needle_input.clone(), h.clone());

                self.haystack_index.fanout_set(h).map_or(false, |fanout| {
                    fanout.iter().all(|d_succ| {
                        next_assignment
                            .get_needle_cell(d_succ.clone())
                            .map_or(true, |p_succ| {
                                self.check_fanin_constraints(
                                    p_succ,
                                    d_succ.clone(),
                                    &next_assignment,
                                )
                            })
                    })
                })
            })
            .collect()
    }

    /// Applies deduplication strategies based on the provided configuration.
    fn apply_deduplication(&self, results: &mut Vec<Embedding<'needle, 'haystack>>) {
        if self.config.dedupe.all() {
            let mut seen = HashSet::new();
            results.retain(|m| seen.insert(m.assignment.signature()));
        }

        if self.config.dedupe.inner() {
            let mut seen = HashSet::new();
            results.retain(|m| seen.insert(m.internal_signature()));
        }
    }
}
