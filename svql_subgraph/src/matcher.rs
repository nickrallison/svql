use std::collections::{HashSet, VecDeque};
use svql_design_set::design_container::DesignContainer;
use tracing::debug;

use prjunnamed_netlist::Design;
use svql_common::Config;
use svql_design_set::cell::{CellKind, CellWrapper};
use svql_design_set::graph_index::GraphIndex;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::embedding::{Embedding, EmbeddingSet};
use crate::mapping::Assignment;
use crate::utils::intersect_sets;

pub struct SubgraphMatcher<'needle, 'haystack, 'cfg> {
    pub(crate) needle: &'needle DesignContainer,
    pub(crate) haystack: &'haystack DesignContainer,
    pub(crate) config: &'cfg Config,
}

impl<'needle, 'haystack, 'cfg> SubgraphMatcher<'needle, 'haystack, 'cfg> {
    pub fn enumerate_embeddings(&self) -> EmbeddingSet<'needle, 'haystack> {
        let (needle_input_mapping_queue, needle_internal_mapping_queue) =
            self.build_needle_work_queues();
        let initial_cell_mapping: Assignment<'needle, 'haystack> = Assignment::new();
        let mut results = self.recurse_internal_cells(
            initial_cell_mapping,
            needle_internal_mapping_queue,
            needle_input_mapping_queue,
            0,
        );

        if self.config.dedupe.all() {
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(m.assignment.signature()));
        }

        if self.config.dedupe.inner() {
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(m.internal_signature()));
        }

        EmbeddingSet {
            items: results,
            needle_input_fanout_by_name: self.needle.index().get_input_fanout_by_name(),
            needle_output_fanin_by_name: self.needle.index().get_output_fanin_by_name(),
        }
    }

    fn recurse_internal_cells(
        &self,
        cell_mapping: Assignment<'needle, 'haystack>,
        mut needle_internal_mapping_queue: VecDeque<CellWrapper<'needle>>,
        needle_input_mapping_queue: VecDeque<CellWrapper<'needle>>,
        recursion_depth: usize,
    ) -> Vec<Embedding<'needle, 'haystack>> {
        // Base Case
        let Some(needle_current) = needle_internal_mapping_queue.pop_front() else {
            return self.recurse_input_cells(
                cell_mapping,
                needle_input_mapping_queue,
                recursion_depth + 1,
            );
        };

        let candidates = self.candidates_for_cell(needle_current.clone(), &cell_mapping);

        #[cfg(feature = "rayon")]
        let candidates_iter = candidates.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let candidates_iter = candidates.into_iter();

        let results = candidates_iter.map(|candidate_cell| {
            let mut next_assignment = cell_mapping.clone();
            next_assignment.assign(needle_current.clone(), candidate_cell.clone());

            self.recurse_internal_cells(
                next_assignment,
                needle_internal_mapping_queue.clone(),
                needle_input_mapping_queue.clone(),
                recursion_depth + 1,
            )
        });

        let embeddings: Vec<Embedding<'needle, 'haystack>> = results.flatten().collect();
        debug!(
            "Depth {} returning {} results",
            recursion_depth,
            embeddings.len()
        );
        embeddings
    }

    fn recurse_input_cells(
        &self,
        cell_mapping: Assignment<'needle, 'haystack>,
        mut needle_input_queue: VecDeque<CellWrapper<'needle>>,
        recursion_depth: usize,
    ) -> Vec<Embedding<'needle, 'haystack>> {
        // Base Case
        let Some(needle_current) = needle_input_queue.pop_front() else {
            let mapping = Embedding {
                assignment: cell_mapping,
            };

            return vec![mapping];
        };

        let candidates_vec = self.candidates_for_input(needle_current.clone(), &cell_mapping);

        #[cfg(feature = "rayon")]
        let candidates_iter = candidates_vec.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let candidates_iter = candidates_vec.into_iter();

        let results = candidates_iter.map(|candidate_cell| {
            let mut nm = cell_mapping.clone();
            nm.assign(needle_current.clone(), candidate_cell.clone());

            self.recurse_input_cells(nm, needle_input_queue.clone(), recursion_depth + 1)
        });

        let flat_results: Vec<Embedding<'needle, 'haystack>> = results.flatten().collect();
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
            .needle_index
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
            .needle_index
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
            self.haystack_index
                .cells_of_type_iter(current_kind)
                .map(|iter| iter.cloned().collect())
                .unwrap_or_default()
        } else {
            let haystack_fan_out_sets: Vec<HashSet<CellWrapper<'haystack>>> = mapped_haystack_fanin
                .iter()
                .filter_map(|haystack_cell| self.haystack_index.fanout_set(haystack_cell))
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
            .needle_index
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
            .filter_map(|haystack_cell| self.haystack_index.fanin_set(haystack_cell))
            .collect();

        let intersection_haystack_fan_in: HashSet<CellWrapper<'haystack>> =
            intersect_sets(haystack_fan_in_sets);

        let candidates: Vec<CellWrapper<'haystack>> = intersection_haystack_fan_in
            .into_iter()
            .filter(|haystack_cell| {
                let mut next_assignment = cell_mapping.clone();
                next_assignment.assign(needle_current.clone(), haystack_cell.clone());

                let fanout = self.haystack_index.fanout_set(haystack_cell);
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
}
