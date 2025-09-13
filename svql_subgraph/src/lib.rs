#![allow(dead_code)]
// mod candidates;
mod connectivity;
mod mapping;

pub mod cell;
pub mod graph_index;

use graph_index::DesignIndex;
use mapping::Mapping;

use prjunnamed_netlist::Design;
use tracing::debug;

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use svql_common::Config;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::cell::{CellKind, CellWrapper};

#[derive(Clone, Debug, Default)]
pub struct AllEmbeddings<'needle, 'haystack> {
    pub embeddings: Vec<Embedding<'needle, 'haystack>>,
    pub needle_input_fanout: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
    pub needle_output_fanin: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
}

#[derive(Clone, Debug, Default)]
pub struct Embedding<'needle, 'haystack> {
    pub mapping: Mapping<'needle, 'haystack>,
}

impl<'needle, 'haystack> Embedding<'needle, 'haystack> {
    pub fn len(&self) -> usize {
        self.mapping.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub fn debug_print(&self) {
        let mapping = self.mapping.needle_mapping();
        for (pat_cell, des_cell) in mapping {
            println!(
                "{}: {:?} -> {}: {:?}",
                pat_cell.debug_index(),
                pat_cell.get(),
                des_cell.debug_index(),
                des_cell.get()
            );
        }
        println!("--------------------------------------------------------")
    }

    pub fn internal_signature(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .mapping
            .needle_mapping()
            .iter()
            .filter(|(p, _)| !matches!(p.cell_type(), CellKind::Input | CellKind::Output))
            .map(|(_, d)| d.debug_index())
            .collect();
        sig.sort_unstable();
        sig
    }
}

pub struct SubgraphMatcher<'needle, 'haystack, 'cfg> {
    needle: &'needle Design,
    haystack: &'haystack Design,
    needle_index: DesignIndex<'needle>,
    haystack_index: DesignIndex<'haystack>,
    config: &'cfg Config,
}

pub(crate) struct SubgraphMatcherCore<'needle, 'haystack, 'cfg> {
    needle: &'needle Design,
    haystack: &'haystack Design,
    needle_index: &'cfg DesignIndex<'needle>,
    haystack_index: &'cfg DesignIndex<'haystack>,
    config: &'cfg Config,
}

impl<'needle, 'haystack, 'cfg> SubgraphMatcher<'needle, 'haystack, 'cfg> {
    pub fn search_all(
        needle: &'needle Design,
        haystack: &'haystack Design,
        config: &'cfg Config,
    ) -> AllEmbeddings<'needle, 'haystack> {
        let needle_index = DesignIndex::build(needle);
        let haystack_index = DesignIndex::build(haystack);

        let matcher = SubgraphMatcherCore {
            needle,
            haystack,
            needle_index: &needle_index,
            haystack_index: &haystack_index,
            config,
        };

        matcher.enumerate_embeddings()
    }

    pub fn search_with_indices(
        needle: &'needle Design,
        haystack: &'haystack Design,
        needle_index: &'cfg DesignIndex<'needle>,
        haystack_index: &'cfg DesignIndex<'haystack>,
        config: &'cfg Config,
    ) -> AllEmbeddings<'needle, 'haystack> {
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
    pub fn enumerate_embeddings(&self) -> AllEmbeddings<'needle, 'haystack> {
        let (needle_input_mapping_queue, needle_gate_mapping_queue) =
            self.build_needle_work_queues();
        let initial_cell_mapping: Mapping<'needle, 'haystack> = Mapping::new();
        let mut results = self.recurse_internal_cells(
            initial_cell_mapping,
            needle_gate_mapping_queue,
            needle_input_mapping_queue,
            0,
        );

        if self.config.dedupe.all() {
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(m.mapping.signature()));
        }

        if self.config.dedupe.inner() {
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(m.internal_signature()));
        }

        AllEmbeddings {
            embeddings: results,
            needle_input_fanout: self.needle_index.get_input_fanout_by_name().clone(),
            needle_output_fanin: self.needle_index.get_output_fanin_by_name().clone(),
        }
    }

    fn recurse_internal_cells(
        &self,
        cell_mapping: Mapping<'needle, 'haystack>,
        mut needle_gate_mapping_queue: VecDeque<CellWrapper<'needle>>,
        needle_input_mapping_queue: VecDeque<CellWrapper<'needle>>,
        recursion_depth: usize,
    ) -> Vec<Embedding<'needle, 'haystack>> {
        // Base Case
        let Some(needle_current) = needle_gate_mapping_queue.pop_front() else {
            return self.recurse_input_cells(
                cell_mapping,
                needle_input_mapping_queue,
                recursion_depth + 1,
            );
        };

        let candidates_vec = self.candidates_for_cell(needle_current.clone(), &cell_mapping);

        #[cfg(feature = "rayon")]
        let cand_iter = candidates_vec.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let cand_iter = candidates_vec.into_iter();

        let results = cand_iter.map(|candidate_cell| {
            let mut nm = cell_mapping.clone();
            nm.insert(needle_current.clone(), candidate_cell.clone());

            self.recurse_internal_cells(
                nm,
                needle_gate_mapping_queue.clone(),
                needle_input_mapping_queue.clone(),
                recursion_depth + 1,
            )
        });

        let flat_results: Vec<Embedding<'needle, 'haystack>> = results.flatten().collect();
        debug!(
            "Depth {} returning {} results",
            recursion_depth,
            flat_results.len()
        );
        flat_results
    }

    fn recurse_input_cells(
        &self,
        cell_mapping: Mapping<'needle, 'haystack>,
        mut needle_work: VecDeque<CellWrapper<'needle>>,
        recursion_depth: usize,
    ) -> Vec<Embedding<'needle, 'haystack>> {
        // Base Case
        let Some(needle_current) = needle_work.pop_front() else {
            let mapping = Embedding {
                mapping: cell_mapping,
            };

            return vec![mapping];
        };

        let candidates_vec = self.candidates_for_input(needle_current.clone(), &cell_mapping);

        #[cfg(feature = "rayon")]
        let cand_iter = candidates_vec.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let cand_iter = candidates_vec.into_iter();

        let results = cand_iter.map(|candidate_cell| {
            let mut nm = cell_mapping.clone();
            nm.insert(needle_current.clone(), candidate_cell.clone());

            self.recurse_input_cells(nm, needle_work.clone(), recursion_depth + 1)
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
        let mut needle_mapping_queue: Vec<CellWrapper<'needle>> = self
            .needle_index
            .topo_cells()
            .iter()
            .filter(|c| !matches!(c.cell_type(), CellKind::Output))
            .cloned()
            .rev()
            .collect();

        let mut pending_input_cells: VecDeque<CellWrapper<'needle>> = VecDeque::new();
        let mut pending_gate_cells: VecDeque<CellWrapper<'needle>> = VecDeque::new();

        for cell in needle_mapping_queue.drain(..) {
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
        cell_mapping: &Mapping<'needle, 'haystack>,
    ) -> Vec<CellWrapper<'haystack>> {
        let current_type = needle_current.cell_type();

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
            self.haystack_index.cells_of_type(current_type).to_vec()
        } else {
            let haystack_fan_out_sets: Vec<HashSet<CellWrapper<'haystack>>> = mapped_haystack_fanin
                .iter()
                .filter_map(|d_cell| self.haystack_index.fanout_set(d_cell))
                .collect();

            let intersection_haystack_fan_out: HashSet<CellWrapper<'haystack>> =
                intersection(haystack_fan_out_sets);
            intersection_haystack_fan_out.into_iter().collect()
        };

        let candidates: Vec<CellWrapper<'haystack>> = unfiltered_candidates
            .into_iter()
            .filter(|d_cell| {
                self.check_fanin_constraints(needle_current.clone(), d_cell.clone(), cell_mapping)
            })
            .filter(|d_cell| d_cell.cell_type() == current_type)
            .filter(|d_cell| cell_mapping.haystack_mapping().get(d_cell).is_none())
            .collect();

        candidates
    }

    fn candidates_for_input(
        &self,
        needle_current: CellWrapper<'needle>,
        cell_mapping: &Mapping<'needle, 'haystack>,
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
            .filter_map(|d_cell| self.haystack_index.fanin_set(d_cell))
            .collect();

        let intersection_haystack_fan_in: HashSet<CellWrapper<'haystack>> =
            intersection(haystack_fan_in_sets);

        let candidates: Vec<CellWrapper<'haystack>> = intersection_haystack_fan_in
            .into_iter()
            .filter(|d_cell| {
                let mut next_assignment = cell_mapping.clone();
                next_assignment.insert(needle_current.clone(), d_cell.clone());

                let fanout = self.haystack_index.fanout_set(d_cell);
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
            .filter(|d_cell| cell_mapping.haystack_mapping().get(d_cell).is_none())
            .collect();

        candidates
    }
}

pub fn intersection<T: Eq + Hash + Clone>(mut items: Vec<HashSet<T>>) -> HashSet<T> {
    let Some(first_fanin) = items.pop() else {
        return HashSet::new();
    };

    let intersection: HashSet<T> = items.iter().fold(first_fanin, |acc: HashSet<T>, hs| {
        acc.intersection(hs).cloned().collect()
    });

    intersection
}

pub fn intersection_ref<T: Eq + Hash + Clone>(mut items: Vec<&HashSet<T>>) -> HashSet<T> {
    let Some(first_fanin) = items.pop() else {
        return HashSet::new();
    };

    let first_fanin = first_fanin.clone();

    let intersection: HashSet<T> = items.iter().fold(first_fanin, |acc: HashSet<T>, hs| {
        acc.intersection(hs).cloned().collect()
    });

    intersection
}
