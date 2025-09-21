use std::collections::{HashSet, VecDeque};
use svql_design_set::DesignSet;
use svql_design_set::design_container::DesignContainer;
use tracing::debug;

use svql_common::Config;
use svql_design_set::cell::{CellKind, CellWrapper};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::mapping::Assignment;
use crate::match_::{Match, Matches, PartialMatch};
use crate::utils::intersect_sets;

pub struct SubgraphMatcher<'needle, 'haystack, 'cfg> {
    pub(crate) needle: &'needle DesignContainer,
    pub(crate) haystack_key: String,
    pub(crate) haystack_set: &'haystack DesignSet,
    pub(crate) config: &'cfg Config,
}

impl<'needle, 'haystack, 'cfg> SubgraphMatcher<'needle, 'haystack, 'cfg> {
    pub fn new(
        needle: &'needle DesignContainer,
        haystack_key: String,
        haystack_set: &'haystack DesignSet,
        config: &'cfg Config,
    ) -> Self {
        Self {
            needle,
            haystack_key,
            haystack_set,
            config,
        }
    }

    pub fn get_matches(&self) -> Matches<'needle, 'haystack> {
        let (needle_input_mapping_queue, needle_internal_mapping_queue) =
            self.build_needle_work_queues();

        let initial_partial_match = PartialMatch {
            assignment: Assignment::new(),
            needle_input_mapping_queue: needle_input_mapping_queue,
            needle_internal_mapping_queue: needle_internal_mapping_queue,
            call_stack: vec![self.haystack_key.clone()],
        };

        let child_partial_matches: Vec<PartialMatch<'needle, 'haystack>> =
            todo!("get partial matches from child modules (recusively)");

        let partial_matches = vec![initial_partial_match]
            .into_iter()
            .chain(child_partial_matches.into_iter())
            .collect::<Vec<PartialMatch<'needle, 'haystack>>>();

        let mut results = partial_matches
            .into_iter()
            .flat_map(|pm| self.get_matches_rec(pm))
            .collect::<Vec<PartialMatch<'needle, 'haystack>>>();

        let mut results = results
            .drain(..)
            .map(|pm| Match {
                assignment: pm.assignment,
            })
            .collect::<Vec<Match<'needle, 'haystack>>>();

        if self.config.dedupe.all() {
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(m.assignment.signature()));
        }

        if self.config.dedupe.inner() {
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(m.internal_signature()));
        }

        Matches {
            items: results,
            needle_input_fanout_by_name: self.needle.index().get_input_fanout_by_name(),
            needle_output_fanin_by_name: self.needle.index().get_output_fanin_by_name(),
        }
    }

    fn get_matches_rec(
        &self,
        mut partial_match: PartialMatch<'needle, 'haystack>,
    ) -> Vec<PartialMatch<'needle, 'haystack>> {
        // Base Case
        let Some(needle_current) = partial_match.needle_internal_mapping_queue.pop_front() else {
            return self.recurse_input_cells(partial_match);
        };

        let candidates =
            self.candidates_for_cell(needle_current.clone(), &partial_match.assignment);

        #[cfg(feature = "rayon")]
        let candidates_iter = candidates.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let candidates_iter = candidates.into_iter();

        let results = candidates_iter.map(|candidate_cell| {
            let mut new_pm: PartialMatch<'needle, 'haystack> = partial_match.clone();
            new_pm
                .assignment
                .assign(needle_current.clone(), candidate_cell.clone());

            self.get_matches_rec(new_pm)
        });

        // let embeddings: Vec<Match<'needle, 'haystack>> = results
        //     .filter_map(|pm| pm.try_into().ok())
        //     .flatten()
        //     .collect();

        let embeddings: Vec<PartialMatch<'needle, 'haystack>> = results.flatten().collect();
        let recursion_depth = partial_match.assignment.len();
        debug!(
            "Depth {} returning {} results",
            recursion_depth,
            embeddings.len()
        );
        embeddings
    }

    fn recurse_input_cells(
        &self,
        mut partial_match: PartialMatch<'needle, 'haystack>,
    ) -> Vec<PartialMatch<'needle, 'haystack>> {
        // Base Case
        let Some(needle_current) = partial_match.needle_input_mapping_queue.pop_front() else {
            return vec![partial_match];
        };

        let candidates_vec =
            self.candidates_for_input(needle_current.clone(), &partial_match.assignment);

        #[cfg(feature = "rayon")]
        let candidates_iter = candidates_vec.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let candidates_iter = candidates_vec.into_iter();

        let results = candidates_iter.map(|candidate_cell| {
            let mut new_pm = partial_match.clone();
            new_pm
                .assignment
                .assign(needle_current.clone(), candidate_cell.clone());

            self.recurse_input_cells(new_pm)
        });

        let flat_results: Vec<PartialMatch<'needle, 'haystack>> = results.flatten().collect();
        let recursion_depth = partial_match.assignment.len();
        debug!(
            "Depth {} returning {} results",
            recursion_depth,
            flat_results.len()
        );
        flat_results
    }

    // ##########################

    fn build_needle_work_queues(
        &self,
    ) -> (
        VecDeque<CellWrapper<'needle>>,
        VecDeque<CellWrapper<'needle>>,
    ) {
        let mut topo_ordered_cells: Vec<CellWrapper<'needle>> = self
            .needle
            .index()
            .cells_topo()
            .iter()
            .filter(|c| !matches!(c.cell_type(), CellKind::Output))
            .cloned()
            .rev()
            .collect();

        let mut pending_input_cells: VecDeque<CellWrapper<'needle>> = VecDeque::new();
        let mut pending_gate_cells: VecDeque<CellWrapper<'needle>> = VecDeque::new();

        for cell in topo_ordered_cells.drain(..) {
            match cell.cell_type() {
                CellKind::Input => pending_input_cells.push_back(cell),
                _ => pending_gate_cells.push_back(cell),
            }
        }

        (pending_input_cells, pending_gate_cells)
    }

    fn candidates_for_cell(
        &self,
        needle_current: CellWrapper<'needle>,
        cell_mapping: &Assignment<'needle, 'haystack>,
    ) -> Vec<CellWrapper<'haystack>> {
        let current_kind = needle_current.cell_type();

        let needle_fan_in = self
            .needle
            .index()
            .fanin_with_ports(&needle_current)
            .unwrap_or_default();

        let mapped_haystack_fanin: Vec<CellWrapper<'haystack>> = needle_fan_in
            .iter()
            .filter_map(|(p_fan_in_cell, _)| cell_mapping.get_haystack_cell(p_fan_in_cell.clone()))
            .collect();

        let unfiltered_candidates: Vec<CellWrapper<'haystack>> = if mapped_haystack_fanin.is_empty()
        {
            // if no fanin mapped, return all cells of the correct type
            // This happens for the first cells mapped and is not avoidable
            self.haystack()
                .index()
                .cells_of_type_iter(current_kind)
                .map(|iter| iter.cloned().collect())
                .unwrap_or_default()
        } else {
            let haystack_fan_out_sets: Vec<HashSet<CellWrapper<'haystack>>> = mapped_haystack_fanin
                .iter()
                .filter_map(|haystack_cell| self.haystack().index().fanout_set(haystack_cell))
                .collect();

            let intersection_haystack_fan_out: HashSet<CellWrapper<'haystack>> =
                intersect_sets(haystack_fan_out_sets);
            intersection_haystack_fan_out.into_iter().collect()
        };

        let candidates: Vec<CellWrapper<'haystack>> = unfiltered_candidates
            .into_iter()
            .filter(|haystack_cell| {
                self.check_fanin_constraints(
                    needle_current.clone(),
                    haystack_cell.clone(),
                    cell_mapping,
                )
            })
            .filter(|haystack_cell| haystack_cell.cell_type() == current_kind)
            .filter(|haystack_cell| cell_mapping.haystack_mapping().get(haystack_cell).is_none())
            .collect();

        candidates
    }

    fn candidates_for_input(
        &self,
        needle_current: CellWrapper<'needle>,
        cell_mapping: &Assignment<'needle, 'haystack>,
    ) -> Vec<CellWrapper<'haystack>> {
        let needle_fan_out = self
            .needle
            .index()
            .fanout_with_ports(&needle_current)
            .unwrap_or_default();

        let mapped_haystack_fan_out: Vec<CellWrapper<'haystack>> = needle_fan_out
            .iter()
            .filter_map(|(p_fan_out_cell, _)| {
                cell_mapping.get_haystack_cell(p_fan_out_cell.clone())
            })
            .collect();

        let haystack_fan_in_sets: Vec<HashSet<CellWrapper<'haystack>>> = mapped_haystack_fan_out
            .iter()
            .filter_map(|haystack_cell| self.haystack().index().fanin_set(haystack_cell))
            .collect();

        let intersection_haystack_fan_in: HashSet<CellWrapper<'haystack>> =
            intersect_sets(haystack_fan_in_sets);

        let candidates: Vec<CellWrapper<'haystack>> = intersection_haystack_fan_in
            .into_iter()
            .filter(|haystack_cell| {
                let mut next_assignment = cell_mapping.clone();
                next_assignment.assign(needle_current.clone(), haystack_cell.clone());

                let fanout = self.haystack().index().fanout_set(haystack_cell);
                if fanout.is_none() {
                    return false;
                }
                let fanout = fanout.unwrap();
                fanout.iter().all(|d_fanout_cell| {
                    if let Some(p_fanout_cell) =
                        next_assignment.get_needle_cell(d_fanout_cell.clone())
                    {
                        self.check_fanin_constraints(
                            p_fanout_cell,
                            d_fanout_cell.clone(),
                            &next_assignment,
                        )
                    } else {
                        true
                    }
                })
            })
            .filter(|haystack_cell| cell_mapping.haystack_mapping().get(haystack_cell).is_none())
            .collect();

        candidates
    }

    pub(crate) fn haystack(&self) -> &'haystack DesignContainer {
        self.haystack_set
            .get(&self.haystack_key)
            .expect("Haystack module not found in design set")
    }
}
