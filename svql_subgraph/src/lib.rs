#![allow(dead_code)]
// mod candidates;
mod cell_mapping;
mod constraints;
mod util;

pub mod cell;
pub mod design_index;

use cell_mapping::CellMapping;
use design_index::DesignIndex;

use prjunnamed_netlist::Design;
use tracing::debug;

use std::collections::{HashSet, VecDeque};
use std::hash::Hash;
use svql_common::Config;

pub use util::*;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::cell::{CellType, CellWrapper};

#[derive(Clone, Debug, Default)]
pub struct SubgraphIsomorphism<'p, 'd> {
    // Mapping of pattern cells to design cells (and reverse)
    pub mapping: CellMapping<'p, 'd>,
    // // Boundary IO lookup tables
    // pub input_by_name: HashMap<&'p str, CellWrapper<'p>>,
    // pub output_by_name: HashMap<&'p str, CellWrapper<'p>>,
    //
}

impl<'p, 'd> SubgraphIsomorphism<'p, 'd> {
    pub fn len(&self) -> usize {
        self.mapping.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub fn print_mapping(&self) {
        let mapping = self.mapping.pattern_mapping();
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
}

pub struct FindSubgraphs<'p, 'd, 'a> {
    pattern: &'p Design,
    design: &'d Design,
    pattern_index: DesignIndex<'p>,
    design_index: DesignIndex<'d>,
    config: &'a Config,
}

impl<'p, 'd, 'a> FindSubgraphs<'p, 'd, 'a> {
    pub fn new(
        pattern: &'p Design,
        design: &'d Design,
        config: &'a Config,
    ) -> FindSubgraphs<'p, 'd, 'a> {
        let pattern_index = DesignIndex::build(pattern);
        let design_index = DesignIndex::build(design);

        FindSubgraphs {
            pattern,
            design,
            pattern_index,
            design_index,
            config,
        }
    }

    pub fn from_index(
        pattern: &'p Design,
        design: &'d Design,
        pattern_index: DesignIndex<'p>,
        design_index: DesignIndex<'d>,
        config: &'a Config,
    ) -> FindSubgraphs<'p, 'd, 'a> {
        FindSubgraphs {
            pattern,
            design,
            pattern_index,
            design_index,
            config,
        }
    }

    pub fn find_subgraph_isomorphisms(&self) -> Vec<SubgraphIsomorphism<'p, 'd>> {
        let (pattern_gate_mapping_queue, pattern_input_mapping_queue) =
            self.build_pattern_mapping_queues();
        let initial_cell_mapping: CellMapping<'p, 'd> = CellMapping::new();
        let mut results = self.find_isomorphisms_recurse(
            initial_cell_mapping,
            pattern_gate_mapping_queue,
            pattern_input_mapping_queue,
            0,
        );

        if self.config.dedupe {
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(m.mapping.signature()));
        }

        results
    }

    fn find_isomorphisms_recurse(
        &self,
        cell_mapping: CellMapping<'p, 'd>,
        mut pattern_gate_mapping_queue: VecDeque<CellWrapper<'p>>,
        mut pattern_input_mapping_queue: VecDeque<CellWrapper<'p>>,
        depth: usize,
    ) -> Vec<SubgraphIsomorphism<'p, 'd>> {
        // Base Case
        let Some(pattern_current) = pattern_gate_mapping_queue.pop_front() else {
            return self.find_isomorphisms_recurse_inputs(
                cell_mapping,
                pattern_input_mapping_queue,
                depth + 1,
            );
        };

        let candidates_vec = self.build_gate_candidates(pattern_current.clone(), &cell_mapping);

        #[cfg(feature = "rayon")]
        let cand_iter = candidates_vec.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let cand_iter = candidates_vec.into_iter();

        let results = cand_iter.map(|d_candidate| {
            let mut nm = cell_mapping.clone();
            nm.insert(pattern_current.clone(), d_candidate.clone());

            self.find_isomorphisms_recurse(
                nm,
                pattern_gate_mapping_queue.clone(),
                pattern_input_mapping_queue.clone(),
                depth + 1,
            )
        });

        let flat_results: Vec<SubgraphIsomorphism<'p, 'd>> = results.flatten().collect();
        debug!("Depth {} returning {} results", depth, flat_results.len());
        flat_results
    }

    fn find_isomorphisms_recurse_inputs(
        &self,
        cell_mapping: CellMapping<'p, 'd>,
        mut pattern_input_mapping_queue: VecDeque<CellWrapper<'p>>,
        depth: usize,
    ) -> Vec<SubgraphIsomorphism<'p, 'd>> {
        // Base Case
        let Some(pattern_current) = pattern_input_mapping_queue.pop_front() else {
            // attach
            let mut mapping = SubgraphIsomorphism {
                mapping: cell_mapping,
                // input_by_name: self.pattern_index.get_input_by_name().clone(),
                // output_by_name: self.pattern_index.get_output_by_name().clone(),
            };

            return vec![mapping];
        };

        let candidates_vec = self.build_input_candidates(pattern_current.clone(), &cell_mapping);

        #[cfg(feature = "rayon")]
        let cand_iter = candidates_vec.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let cand_iter = candidates_vec.into_iter();

        let results = cand_iter.map(|d_candidate| {
            let mut nm = cell_mapping.clone();
            nm.insert(pattern_current.clone(), d_candidate.clone());

            self.find_isomorphisms_recurse_inputs(
                nm,
                pattern_input_mapping_queue.clone(),
                depth + 1,
            )
        });

        let flat_results: Vec<SubgraphIsomorphism<'p, 'd>> = results.flatten().collect();
        debug!("Depth {} returning {} results", depth, flat_results.len());
        flat_results
    }

    // ##########################

    fn build_pattern_mapping_queues(
        &self,
    ) -> (VecDeque<CellWrapper<'p>>, VecDeque<CellWrapper<'p>>) {
        let mut pattern_mapping_queue: Vec<CellWrapper<'p>> = self
            .pattern_index
            .get_cells_topo()
            .iter()
            .filter(|c| !matches!(c.cell_type(), CellType::Output))
            .cloned()
            .rev()
            .collect();

        let mut pattern_input_mapping_queue: VecDeque<CellWrapper<'p>> = VecDeque::new();
        let mut pattern_gate_mapping_queue: VecDeque<CellWrapper<'p>> = VecDeque::new();

        for cell in pattern_mapping_queue.drain(..) {
            match cell.cell_type() {
                CellType::Input => pattern_input_mapping_queue.push_back(cell),
                _ => pattern_gate_mapping_queue.push_back(cell),
            }
        }

        (pattern_input_mapping_queue, pattern_gate_mapping_queue)
    }

    fn build_gate_candidates(
        &self,
        pattern_current: CellWrapper<'p>,
        cell_mapping: &CellMapping<'p, 'd>,
    ) -> Vec<CellWrapper<'d>> {
        let current_type = pattern_current.cell_type();

        // The candidates all must be the correct type based on the input cell
        // let candidates = {
        //     let slice: &'a [CellWrapper<'d>] = match current_type {
        //         CellType::Input => design_index.get_cells_topo(),
        //         _ => design_index.get_by_type(current_type),
        //     };

        //     #[cfg(feature = "rayon")]
        //     let slice_iter = slice.into_par_iter();

        //     #[cfg(not(feature = "rayon"))]
        //     let slice_iter = slice.into_iter();

        //     slice_iter
        // };

        let pattern_fan_in = self
            .pattern_index
            .get_fanin_raw(&pattern_current)
            .map(|vec| vec.as_slice())
            .unwrap_or_default();

        let mapped_design_fan_in: Vec<CellWrapper<'d>> = pattern_fan_in
            .iter()
            .filter_map(|(p_fan_in_cell, _)| cell_mapping.get_design_cell(p_fan_in_cell.clone()))
            .collect();

        let design_fan_out_sets: Vec<HashSet<CellWrapper<'d>>> = mapped_design_fan_in
            .iter()
            .map(|d_cell| self.design_index.get_fanout(d_cell))
            .collect();

        let intersection_design_fan_out: HashSet<CellWrapper<'d>> =
            intersection(design_fan_out_sets);

        let candidates: Vec<CellWrapper<'d>> = intersection_design_fan_out
            .into_iter()
            .filter(|d_cell| {
                self.validate_fan_in_connections(
                    pattern_current.clone(),
                    d_cell.clone(),
                    cell_mapping,
                )
            })
            .filter(|d_cell| d_cell.cell_type() == current_type)
            .filter(|d_cell| cell_mapping.design_mapping().get(d_cell).is_none())
            .collect();

        candidates

        // // Filter 1: Filter only cells that have fan out from mapped design cells
        // // This is to cut down the number of possible candidates to search
        // // let fan_out_from_mapped_design: Option<HashSet<CellWrapper<'d>>> = None;

        // // Filter 2: Filter only not already mapped cells
        // let not_already_mapped_filter: NotAlreadyMappedConstraint<'p, 'd> =
        //     NotAlreadyMappedConstraint::new(cell_mapping.clone());

        // // Filter 3: If that cell is chosen as a mapping for pattern, it must not invalidate the connectivity specified by by the pattern
        // // since cells are chosen in the order inputs -> outputs
        // // we check that for each design cell <-> pattern cell, their fan in are connected (since in topological order)

        // candidates
        //     .filter(|d_candidate| not_already_mapped_filter.d_candidate_is_valid(d_candidate))
        //     .filter(|d_candidate| connectivity_filter.d_candidate_is_valid(d_candidate))
        //     .cloned()
        //     .collect()
    }

    fn build_input_candidates(
        &self,
        pattern_current: CellWrapper<'p>,
        cell_mapping: &CellMapping<'p, 'd>,
    ) -> Vec<CellWrapper<'d>> {
        let pattern_fan_out = self
            .pattern_index
            .get_fanout_raw(&pattern_current)
            .map(|vec| vec.as_slice())
            .unwrap_or_default();

        let mapped_design_fan_out: Vec<CellWrapper<'d>> = pattern_fan_out
            .iter()
            .filter_map(|(p_fan_out_cell, _)| cell_mapping.get_design_cell(p_fan_out_cell.clone()))
            .collect();

        let design_fan_in_sets: Vec<HashSet<CellWrapper<'d>>> = mapped_design_fan_out
            .iter()
            .map(|d_cell| self.design_index.get_fanin(d_cell))
            .collect();

        let intersection_design_fan_in: HashSet<CellWrapper<'d>> = intersection(design_fan_in_sets);

        let candidates: Vec<CellWrapper<'d>> = intersection_design_fan_in
            .into_iter()
            .filter(|d_cell| {
                // self.validate_fan_in_connections(
                //     pattern_current.clone(),
                //     d_cell.clone(),
                //     cell_mapping,
                // )
                // #########
                // validate fanout connections
                let mut nm = cell_mapping.clone();
                nm.insert(pattern_current.clone(), d_cell.clone());

                let fanout = self.design_index.get_fanout(d_cell);
                fanout.iter().all(|d_fanout_cell| {
                    if let Some(p_fanout_cell) = nm.get_pattern_cell(d_fanout_cell.clone()) {
                        self.validate_fan_in_connections(p_fanout_cell, d_fanout_cell.clone(), &nm)
                    } else {
                        true
                    }
                })
            })
            // .filter(|d_cell| d_cell.cell_type() == current_type)
            .filter(|d_cell| cell_mapping.design_mapping().get(d_cell).is_none())
            .collect();

        candidates

        // // Filter 1: Filter only cells that have fan out from mapped design cells
        // // This is to cut down the number of possible candidates to search
        // // let fan_out_from_mapped_design: Option<HashSet<CellWrapper<'d>>> = None;

        // // Filter 2: Filter only not already mapped cells
        // let not_already_mapped_filter: NotAlreadyMappedConstraint<'p, 'd> =
        //     NotAlreadyMappedConstraint::new(cell_mapping.clone());

        // // Filter 3: If that cell is chosen as a mapping for pattern, it must not invalidate the connectivity specified by by the pattern
        // // since cells are chosen in the order inputs -> outputs
        // // we check that for each design cell <-> pattern cell, their fan in are connected (since in topological order)

        // candidates
        //     .filter(|d_candidate| not_already_mapped_filter.d_candidate_is_valid(d_candidate))
        //     .filter(|d_candidate| connectivity_filter.d_candidate_is_valid(d_candidate))
        //     .cloned()
        //     .collect()
    }
}

pub fn intersection<T: Eq + Hash + Clone>(mut items: Vec<HashSet<T>>) -> HashSet<T> {
    let Some(first_fanin) = items.pop() else {
        return HashSet::new();
    };

    let intersection: HashSet<T> = items.iter().fold(first_fanin, |acc: HashSet<T>, hs| {
        acc.intersection(&hs).cloned().collect()
    });

    intersection
}
