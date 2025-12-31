//! Subgraph isomorphism matching implementation.
//!
//! This module provides the core matching engine that identifies instances of a
//! needle design within a larger haystack design. It uses a
//! backtracking search algorithm with topological ordering
//! and fan-in/fan-out constraints.

use std::collections::{HashSet, VecDeque};

use prjunnamed_netlist::Design;
use svql_common::Config;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::assignment::{AssignmentSet, SingleAssignment};
use crate::cell::{CellKind, CellWrapper};
use crate::graph_index::GraphIndex;
use crate::utils::intersect_sets;

/// Entry point for subgraph isomorphism searches.
pub struct SubgraphMatcher<'needle, 'haystack, 'cfg> {
    /// The pattern design to search for.
    needle: &'needle Design,
    /// The design to search within.
    haystack: &'haystack Design,
    /// Structural index for the needle design.
    needle_index: GraphIndex<'needle>,
    /// Structural index for the haystack design.
    haystack_index: GraphIndex<'haystack>,
    /// Configuration settings for the matcher.
    pub(crate) config: &'cfg Config,
}

/// Core implementation of the subgraph matching algorithm.
pub struct SubgraphMatcherCore<'needle, 'haystack, 'cfg> {
    /// The pattern design to search for.
    pub(crate) needle: &'needle Design,
    /// The design to search within.
    pub(crate) haystack: &'haystack Design,
    /// Reference to the structural index for the needle.
    pub(crate) needle_index: &'cfg GraphIndex<'needle>,
    /// Reference to the structural index for the haystack.
    pub(crate) haystack_index: &'cfg GraphIndex<'haystack>,
    /// Configuration settings for the matcher.
    pub(crate) config: &'cfg Config,
}

impl<'needle, 'haystack, 'cfg> SubgraphMatcher<'needle, 'haystack, 'cfg> {
    /// Performs a complete subgraph isomorphism search between the needle and haystack.
    ///
    /// This method builds the necessary indices and executes the matching algorithm.
    pub fn enumerate_all(
        needle: &'needle Design,
        haystack: &'haystack Design,
        config: &'cfg Config,
    ) -> AssignmentSet<'needle, 'haystack> {
        let needle_index = GraphIndex::build(needle);
        let haystack_index = GraphIndex::build(haystack);

        let matcher = SubgraphMatcherCore {
            needle,
            haystack,
            needle_index: &needle_index,
            haystack_index: &haystack_index,
            config,
        };

        matcher.enumerate_assignments()
    }

    /// Performs a subgraph isomorphism search using pre-built indices.
    ///
    /// This is more efficient when performing multiple searches against the same designs.
    pub fn enumerate_with_indices(
        needle: &'needle Design,
        haystack: &'haystack Design,
        needle_index: &'cfg GraphIndex<'needle>,
        haystack_index: &'cfg GraphIndex<'haystack>,
        config: &'cfg Config,
    ) -> AssignmentSet<'needle, 'haystack> {
        let matcher = SubgraphMatcherCore {
            needle,
            haystack,
            needle_index,
            haystack_index,
            config,
        };
        matcher.enumerate_assignments()
    }
}

impl<'needle, 'haystack, 'cfg> SubgraphMatcherCore<'needle, 'haystack, 'cfg> {
    /// Executes the subgraph matching process and returns the set of all found mappings.
    pub fn enumerate_assignments(&self) -> AssignmentSet<'needle, 'haystack> {
        tracing::info!(
            "starting subgraph search: needle cells: {}, haystack cells: {}",
            self.needle_index.num_cells(),
            self.haystack_index.num_cells()
        );

        let (input_queue, gate_queue) = self.prepare_search_queues();
        let initial_assignment = SingleAssignment::new();

        let mut results = self.match_gate_cells(initial_assignment, gate_queue, input_queue);

        let total_found = results.len();

        self.apply_deduplication(&mut results);

        tracing::info!(
            "search complete: found {} raw matches, {} after deduplication",
            total_found,
            results.len()
        );

        AssignmentSet {
            items: results,
            needle_input_fanout_by_name: self.needle_index.get_input_fanout_by_name(),
            needle_output_fanin_by_name: self.needle_index.get_output_fanin_by_name(),
        }
    }

    /// Recursively matches gate cells from the needle to candidates in the haystack.
    fn match_gate_cells(
        &self,
        assignment: SingleAssignment<'needle, 'haystack>,
        mut gate_queue: VecDeque<CellWrapper<'needle>>,
        input_queue: VecDeque<CellWrapper<'needle>>,
    ) -> Vec<SingleAssignment<'needle, 'haystack>> {
        let Some(current_needle) = gate_queue.pop_front() else {
            return self.match_input_cells(assignment, input_queue);
        };

        let candidates = self.find_candidates_for_cell(current_needle.clone(), &assignment);

        if candidates.is_empty() {
            tracing::trace!(
                "backtracking: no candidates for needle cell {} ({:?})",
                current_needle.debug_index(),
                current_needle.cell_type()
            );
            return vec![];
        }

        tracing::debug!(
            "needle cell {} ({:?}): found {} candidates",
            current_needle.debug_index(),
            current_needle.cell_type(),
            candidates.len()
        );

        #[cfg(feature = "rayon")]
        let iter = candidates.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = candidates.into_iter();

        iter.flat_map(|candidate| {
            let mut next_assignment = assignment.clone();
            next_assignment.assign(current_needle.clone(), candidate);

            self.match_gate_cells(next_assignment, gate_queue.clone(), input_queue.clone())
        })
        .collect()
    }

    /// Recursively matches input cells from the needle to candidates in the haystack.
    fn match_input_cells(
        &self,
        assignment: SingleAssignment<'needle, 'haystack>,
        mut input_queue: VecDeque<CellWrapper<'needle>>,
    ) -> Vec<SingleAssignment<'needle, 'haystack>> {
        let Some(current_needle) = input_queue.pop_front() else {
            return vec![assignment];
        };

        let candidates = self.find_candidates_for_input(current_needle.clone(), &assignment);

        if candidates.is_empty() && self.config.pattern_vars_match_design_consts {
            return self.match_input_cells(assignment, input_queue);
        }

        #[cfg(feature = "rayon")]
        let iter = candidates.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = candidates.into_iter();

        iter.flat_map(|candidate| {
            let mut next_assignment = assignment.clone();
            next_assignment.assign(current_needle.clone(), candidate);

            self.match_input_cells(next_assignment, input_queue.clone())
        })
        .collect()
    }

    /// Prepares queues for inputs and logic gates.
    ///
    /// Returns a tuple containing `(input_queue, gate_queue)`.
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

    /// Identifies candidate haystack cells that could match a given needle gate cell.
    fn find_candidates_for_cell(
        &self,
        needle_cell: CellWrapper<'needle>,
        assignment: &SingleAssignment<'needle, 'haystack>,
    ) -> Vec<CellWrapper<'haystack>> {
        let kind = needle_cell.cell_type();
        let needle_fanin = self
            .needle_index
            .fanin_with_ports(&needle_cell)
            .unwrap_or_default();

        let mapped_haystack_fanin: Vec<_> = needle_fanin
            .iter()
            .filter_map(|(needle_pred, _)| assignment.get_haystack_cell(needle_pred.clone()))
            .collect();

        let unfiltered: Vec<CellWrapper<'haystack>> = if mapped_haystack_fanin.is_empty() {
            // When no inputs are yet mapped, consider all haystack cells of the matching type.
            self.haystack_index
                .cells_of_type_iter(kind)
                .map(|i| i.cloned().collect())
                .unwrap_or_default()
        } else {
            // When inputs are mapped, candidates are restricted to the intersection of their fan-outs.
            let fanout_sets: Vec<_> = mapped_haystack_fanin
                .iter()
                .filter_map(|haystack_pred| self.haystack_index.fanout_set(haystack_pred))
                .collect();
            intersect_sets(fanout_sets).into_iter().collect()
        };

        unfiltered
            .into_iter()
            .filter(|candidate| candidate.cell_type() == kind)
            .filter(|candidate| assignment.get_needle_cell(candidate.clone()).is_none())
            .filter(|candidate| {
                self.check_fanin_constraints(needle_cell.clone(), candidate.clone(), assignment)
            })
            .collect()
    }

    /// Identifies candidate haystack cells that could match a given needle input.
    fn find_candidates_for_input(
        &self,
        needle_input: CellWrapper<'needle>,
        assignment: &SingleAssignment<'needle, 'haystack>,
    ) -> Vec<CellWrapper<'haystack>> {
        let needle_fanout = self
            .needle_index
            .fanout_with_ports(&needle_input)
            .unwrap_or_default();

        let mapped_haystack_fanout: Vec<_> = needle_fanout
            .iter()
            .filter_map(|(needle_succ, _)| assignment.get_haystack_cell(needle_succ.clone()))
            .collect();

        let fanin_sets: Vec<_> = mapped_haystack_fanout
            .iter()
            .filter_map(|haystack_succ| self.haystack_index.fanin_set(haystack_succ))
            .collect();

        intersect_sets(fanin_sets)
            .into_iter()
            .filter(|candidate| {
                let mut next_assignment = assignment.clone();
                next_assignment.assign(needle_input.clone(), candidate.clone());

                self.haystack_index
                    .fanout_set(candidate)
                    .map_or(false, |fanout| {
                        fanout.iter().all(|haystack_succ| {
                            next_assignment
                                .get_needle_cell(haystack_succ.clone())
                                .map_or(true, |needle_succ| {
                                    self.check_fanin_constraints(
                                        needle_succ,
                                        haystack_succ.clone(),
                                        &next_assignment,
                                    )
                                })
                        })
                    })
            })
            .collect()
    }

    /// Removes duplicate assignments based on the configured deduplication strategy.
    fn apply_deduplication(&self, results: &mut Vec<SingleAssignment<'needle, 'haystack>>) {
        if self.config.dedupe.all() {
            let mut seen = HashSet::new();
            results.retain(|m| seen.insert(m.signature()));
        }

        if self.config.dedupe.inner() {
            let mut seen = HashSet::new();
            results.retain(|m| seen.insert(m.internal_signature()));
        }
    }
}
