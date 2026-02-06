//! Subgraph isomorphism matching implementation.
//!
//! Identifies instances of a needle design within a larger haystack design
//! using a backtracking search algorithm with topological ordering.

use ahash::AHashSet;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

use prjunnamed_netlist::Design;
use svql_common::Config;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::assignment::{AssignmentSet, SingleAssignment};
use crate::cell::{CellKind, CellWrapper};
use crate::graph_index::GraphIndex;
use crate::utils::intersect_sets;

/// Entry point for subgraph isomorphism searches.
#[allow(dead_code)]
pub struct SubgraphMatcher<'needle, 'haystack, 'cfg> {
    needle: &'needle Design,
    haystack: &'haystack Design,
    needle_index: GraphIndex<'needle>,
    haystack_index: GraphIndex<'haystack>,
    pub(crate) config: &'cfg Config,
    needle_name: String,
    haystack_name: String,
}

/// Core implementation of the subgraph matching algorithm with progress tracking.
pub struct SubgraphMatcherCore<'needle, 'haystack, 'cfg> {
    pub(crate) needle: &'needle Design,
    pub(crate) haystack: &'haystack Design,
    pub(crate) needle_index: &'cfg GraphIndex<'needle>,
    pub(crate) haystack_index: &'cfg GraphIndex<'haystack>,
    pub(crate) config: &'cfg Config,
    pub(crate) needle_name: String,
    pub(crate) haystack_name: String,
    pub(crate) branches_explored: AtomicUsize,
    pub(crate) active_branches: AtomicUsize,
    pub(crate) matches_found: AtomicUsize,
    pub(crate) initial_candidates_total: AtomicUsize,
    pub(crate) initial_candidates_done: AtomicUsize,
}

impl<'needle, 'haystack, 'cfg> SubgraphMatcher<'needle, 'haystack, 'cfg> {
    /// Performs a complete subgraph isomorphism search. Builds indices internally.
    #[must_use]
    pub fn enumerate_all(
        needle: &'needle Design,
        haystack: &'haystack Design,
        needle_name: String,
        haystack_name: String,
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
            needle_name,
            haystack_name,
            branches_explored: AtomicUsize::new(0),
            active_branches: AtomicUsize::new(0),
            matches_found: AtomicUsize::new(0),
            initial_candidates_total: AtomicUsize::new(0),
            initial_candidates_done: AtomicUsize::new(0),
        };

        matcher.enumerate_assignments()
    }

    /// Performs a subgraph isomorphism search using pre-built indices.
    #[must_use]
    pub fn enumerate_with_indices(
        needle: &'needle Design,
        haystack: &'haystack Design,
        needle_index: &'cfg GraphIndex<'needle>,
        haystack_index: &'cfg GraphIndex<'haystack>,
        needle_name: String,
        haystack_name: String,
        config: &'cfg Config,
    ) -> AssignmentSet<'needle, 'haystack> {
        let matcher = SubgraphMatcherCore {
            needle,
            haystack,
            needle_index,
            haystack_index,
            config,
            needle_name,
            haystack_name,
            branches_explored: AtomicUsize::new(0),
            active_branches: AtomicUsize::new(0),
            matches_found: AtomicUsize::new(0),
            initial_candidates_total: AtomicUsize::new(0),
            initial_candidates_done: AtomicUsize::new(0),
        };
        matcher.enumerate_assignments()
    }
}

impl<'needle, 'haystack> SubgraphMatcherCore<'needle, 'haystack, '_> {
    /// Executes the matching process and applies deduplication.
    pub fn enumerate_assignments(&self) -> AssignmentSet<'needle, 'haystack> {
        tracing::info!(
            "[{} -> {}] starting subgraph search: needle cells: {}, haystack cells: {}",
            self.needle_name,
            self.haystack_name,
            self.needle_index.num_cells(),
            self.haystack_index.num_cells()
        );

        let (input_queue, gate_queue, output_queue) = self.prepare_search_queues();
        let initial_assignment = SingleAssignment::new();

        let mut results =
            self.match_gate_cells(initial_assignment, gate_queue, input_queue, output_queue);

        let total_found = results.len();

        self.apply_deduplication(&mut results);

        tracing::info!(
            "[{} -> {}] search complete: found {} raw matches, {} after deduplication",
            self.needle_name,
            self.haystack_name,
            total_found,
            results.len()
        );

        AssignmentSet {
            items: results,
            needle_input_fanout_by_name: self.needle_index.get_input_fanout_by_name(),
            needle_output_fanin_by_name: self.needle_index.get_output_fanin_by_name(),
        }
    }

    /// Recursive backtracking step for matching logic gates.
    fn match_gate_cells(
        &self,
        assignment: SingleAssignment<'needle, 'haystack>,
        mut gate_queue: VecDeque<CellWrapper<'needle>>,
        input_queue: VecDeque<CellWrapper<'needle>>,
        output_queue: VecDeque<CellWrapper<'needle>>,
    ) -> Vec<SingleAssignment<'needle, 'haystack>> {
        let total = self.branches_explored.fetch_add(1, Ordering::Relaxed);
        let is_root = assignment.is_empty();

        if total.is_multiple_of(512) && total > 0 {
            let active = self.active_branches.load(Ordering::Relaxed);
            let found = self.matches_found.load(Ordering::Relaxed);
            let top_done = self.initial_candidates_done.load(Ordering::Relaxed);
            let top_total = self.initial_candidates_total.load(Ordering::Relaxed);

            tracing::info!(
                "[{:^10} -> {:^10}] {:>8} branches | {:>3} active | {:>4} matches | Top: {}/{}",
                self.needle_name,
                self.haystack_name,
                total,
                active,
                found,
                top_done,
                top_total
            );
        }

        let Some(current_needle) = gate_queue.pop_front() else {
            return self.match_input_cells(assignment, input_queue, output_queue);
        };

        let candidates = self.find_candidates_for_cell(current_needle.clone(), &assignment);

        if candidates.is_empty() {
            if is_root {
                tracing::warn!("[{}] no candidates found for root cell", self.needle_name);
            }
            return vec![];
        }

        if is_root {
            self.initial_candidates_total
                .store(candidates.len(), Ordering::SeqCst);
        }

        self.active_branches.fetch_add(1, Ordering::SeqCst);

        #[cfg(feature = "rayon")]
        let iter = candidates.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = candidates.into_iter();

        let results: Vec<_> = iter
            .flat_map(|candidate| {
                let mut next_assignment = assignment.clone();
                next_assignment.assign(current_needle.clone(), candidate);

                let res = self.match_gate_cells(
                    next_assignment,
                    gate_queue.clone(),
                    input_queue.clone(),
                    output_queue.clone(),
                );

                if is_root {
                    self.initial_candidates_done.fetch_add(1, Ordering::SeqCst);
                }
                res
            })
            .collect();

        self.active_branches.fetch_sub(1, Ordering::SeqCst);
        results
    }

    /// Recursive backtracking step for matching input ports.
    fn match_input_cells(
        &self,
        assignment: SingleAssignment<'needle, 'haystack>,
        mut input_queue: VecDeque<CellWrapper<'needle>>,
        output_queue: VecDeque<CellWrapper<'needle>>,
    ) -> Vec<SingleAssignment<'needle, 'haystack>> {
        let Some(current_needle) = input_queue.pop_front() else {
            return self.match_output_cells(assignment, output_queue);
        };

        let candidates = self.find_candidates_for_input(current_needle.clone(), &assignment);

        if candidates.is_empty() && self.config.pattern_vars_match_design_consts {
            return self.match_input_cells(assignment, input_queue, output_queue);
        }

        #[cfg(feature = "rayon")]
        let iter = candidates.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = candidates.into_iter();

        iter.flat_map(|candidate| {
            let mut next_assignment = assignment.clone();
            next_assignment.assign(current_needle.clone(), candidate);

            self.match_input_cells(next_assignment, input_queue.clone(), output_queue.clone())
        })
        .collect()
    }

    /// Recursive backtracking step for matching output ports.
    fn match_output_cells(
        &self,
        assignment: SingleAssignment<'needle, 'haystack>,
        mut output_queue: VecDeque<CellWrapper<'needle>>,
    ) -> Vec<SingleAssignment<'needle, 'haystack>> {
        let Some(current_needle) = output_queue.pop_front() else {
            self.matches_found.fetch_add(1, Ordering::Relaxed);
            return vec![assignment];
        };

        let candidates = self.find_candidates_for_output(current_needle.clone(), &assignment);

        if candidates.is_empty() {
            return vec![];
        }

        #[cfg(feature = "rayon")]
        let iter = candidates.into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = candidates.into_iter();

        iter.flat_map(|candidate| {
            let mut next_assignment = assignment.clone();
            next_assignment.assign(current_needle.clone(), candidate);

            self.match_output_cells(next_assignment, output_queue.clone())
        })
        .collect()
    }

    /// Filters haystack cells for output ports based on fan-in connectivity.
    /// Output cells in the pattern can match any logic gate in the haystack.
    fn find_candidates_for_output(
        &self,
        needle_output: CellWrapper<'needle>,
        assignment: &SingleAssignment<'needle, 'haystack>,
    ) -> Vec<CellWrapper<'haystack>> {
        let needle_fanin = self
            .needle_index
            .fanin_with_ports(&needle_output)
            .unwrap_or_default();

        let mapped_haystack_fanin: Vec<_> = needle_fanin
            .iter()
            .filter_map(|(needle_pred, _)| assignment.get_haystack_cell(needle_pred))
            .collect();

        if mapped_haystack_fanin.is_empty() {
            // If no fanin is mapped yet, allow matching to any logic gate
            return self
                .haystack_index
                .cells_topo()
                .iter()
                .filter(|cell| cell.cell_type().is_logic_gate())
                .filter(|candidate| assignment.get_needle_cell(candidate).is_empty())
                .cloned()
                .collect();
        }

        let fanout_sets: Vec<_> = mapped_haystack_fanin
            .iter()
            .filter_map(|haystack_pred| self.haystack_index.fanout_set(haystack_pred))
            .collect();

        intersect_sets(fanout_sets)
            .into_iter()
            .filter(|candidate| assignment.get_needle_cell(candidate).is_empty())
            .collect()
    }

    /// Separates needle cells into input, output, and gate queues for topological traversal.
    fn prepare_search_queues(
        &self,
    ) -> (
        VecDeque<CellWrapper<'needle>>,
        VecDeque<CellWrapper<'needle>>,
        VecDeque<CellWrapper<'needle>>,
    ) {
        let mut inputs = VecDeque::new();
        let mut gates = VecDeque::new();
        let mut outputs = VecDeque::new();

        for cell in self.needle_index.cells_topo().iter().rev() {
            match cell.cell_type() {
                CellKind::Output => outputs.push_back(cell.clone()),
                CellKind::Input => inputs.push_back(cell.clone()),
                _ => gates.push_back(cell.clone()),
            }
        }

        (inputs, gates, outputs)
    }

    /// Filters haystack cells based on type and fan-in constraints.
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
            .filter_map(|(needle_pred, _)| assignment.get_haystack_cell(needle_pred))
            .collect();

        let unfiltered: Vec<CellWrapper<'haystack>> = if mapped_haystack_fanin.is_empty() {
            self.haystack_index
                .cells_of_type_iter(kind)
                .map(|i| i.cloned().collect())
                .unwrap_or_default()
        } else {
            let fanout_sets: Vec<_> = mapped_haystack_fanin
                .iter()
                .filter_map(|haystack_pred| self.haystack_index.fanout_set(haystack_pred))
                .collect();
            intersect_sets(fanout_sets).into_iter().collect()
        };

        unfiltered
            .into_iter()
            .filter(|candidate| candidate.cell_type() == kind)
            .filter(|candidate| assignment.get_needle_cell(candidate).is_empty())
            .filter(|candidate| {
                self.check_fanin_constraints(needle_cell.clone(), candidate.clone(), assignment)
            })
            .collect()
    }

    /// Filters haystack cells for input ports based on fan-out connectivity.
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
            .filter_map(|(needle_succ, _)| assignment.get_haystack_cell(needle_succ))
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
                    .is_some_and(|fanout| {
                        fanout.iter().all(|haystack_succ| {
                            if next_assignment.get_needle_cell(haystack_succ).is_empty() {
                                true
                            } else {
                                next_assignment
                                    .get_needle_cell(haystack_succ)
                                    .into_iter()
                                    .all(|needle_succ| {
                                        self.check_fanin_constraints(
                                            needle_succ.clone(),
                                            haystack_succ.clone(),
                                            &next_assignment,
                                        )
                                    })
                            }
                        })
                    })
            })
            .collect()
    }

    /// Removes duplicate assignments automatically.
    fn apply_deduplication(&self, results: &mut Vec<SingleAssignment<'needle, 'haystack>>) {
        let mut seen = AHashSet::new();
        results.retain(|assignment| seen.insert(assignment.signature()));
    }
}
