#![allow(dead_code)]
// mod candidates;
mod cell_mapping;
mod constraints;
mod graph_index;
mod util;

pub mod cell;

use cell_mapping::CellMapping;
use graph_index::GraphIndex;

use prjunnamed_netlist::{CellRef, Design, Trit};
use tracing::{debug, info, trace};

use std::collections::{HashMap, HashSet, VecDeque};
use svql_common::Config;

pub use util::*;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::{
    cell::{CellType, CellWrapper},
    constraints::{ConnectivityConstraint, Constraint, NotAlreadyMappedConstraint},
};

#[derive(Clone, Debug, Default)]
pub struct SubgraphIsomorphism<'p, 'd> {
    // Mapping of pattern cells to design cells (and reverse)
    mapping: CellMapping<'p, 'd>,

    // Boundary IO lookup tables
    pub input_by_name: HashMap<&'p str, CellWrapper<'p>>,
    pub output_by_name: HashMap<&'p str, CellWrapper<'p>>,
    // For each named output in the pattern, the driver cell/bit in the design for each bit
    // index of that output (usually single-bit in our patterns). Indexed by output name, then bit.
    // output_driver_by_name: HashMap<&'p str, Vec<(CellRef<'d>, usize)>>,
}

impl<'p, 'd> SubgraphIsomorphism<'p, 'd> {
    pub fn len(&self) -> usize {
        self.mapping.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    // pub fn design_source_of_input_bit(
    //     &self,
    //     name: &str,
    //     bit: usize,
    // ) -> Option<(CellRef<'d>, usize)> {
    //     let p_input = self.input_by_name.get(name)?;
    //     let d_src = self.mapping.get_design_cell(*p_input)?;
    //     Some((d_src, bit))
    // }

    // pub fn design_driver_of_output_bit(
    //     &self,
    //     name: &str,
    //     bit: usize,
    // ) -> Option<(CellRef<'d>, usize)> {
    //     self.output_driver_by_name
    //         .get(name)
    //         .and_then(|v| v.get(bit).copied())
    // }

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

/// New API that updates the provided `progress` atomically as the search proceeds.
pub fn find_subgraph_isomorphisms<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    info!("Starting subgraph isomorphism search");
    trace!("Config: {:?}", config);

    let pattern_index = GraphIndex::build(pattern);
    let design_index = GraphIndex::build(design);

    // in topological_order, only gates & inputs (push inputs to the back)
    let pattern_mapping_queue = build_pattern_mapping_queue(&pattern_index);

    let initial_cell_mapping: CellMapping<'p, 'd> = CellMapping::new();

    let mut results = find_isomorphisms_recursive_collect(
        &pattern_index,
        &design_index,
        pattern,
        design,
        config,
        initial_cell_mapping,
        pattern_mapping_queue,
        0, // depth
    );

    info!(
        "Found {} initial results before deduplication",
        results.len()
    );

    if config.dedupe {
        let mut seen: HashSet<Vec<usize>> = HashSet::new();
        results.retain(|m| seen.insert(m.mapping.signature()));
        info!("After AutoMorph deduplication: {} results", results.len());
    }

    // let mut seen: HashSet<Vec<usize>> = HashSet::new();
    // results.retain(|m| seen.insert(m.mapping.signature()));

    info!("Final result count: {}", results.len());

    for result in &results {
        result.print_mapping();
    }

    results
}

fn build_pattern_mapping_queue<'p>(pattern_index: &GraphIndex<'p>) -> VecDeque<CellWrapper<'p>> {
    let pattern_mapping_queue: VecDeque<CellWrapper<'p>> = {
        let q: Vec<CellWrapper<'p>> = pattern_index
            .get_cells_topo()
            .iter()
            // .filter(|c| !matches!(c.cell_type(), CellType::Input))
            .filter(|c| !matches!(c.cell_type(), CellType::Output))
            .cloned()
            .rev()
            .collect();

        q.into()
    };
    pattern_mapping_queue
}

fn build_candidates<'a, 'p, 'd, 'g>(
    pattern_current: CellWrapper<'p>,
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    pattern: &'p Design,
    design: &'d Design,
    config: &'a Config,
    cell_mapping: &CellMapping<'p, 'd>,
) -> Vec<CellWrapper<'d>> {
    let current_type = pattern_current.cell_type();

    // The candidates all must be the correct type based on the input cell
    let candidates = {
        let slice: &'a [CellWrapper<'d>] = match current_type {
            CellType::Input => design_index.get_cells_topo(),
            _ => design_index.get_by_type(current_type),
        };

        #[cfg(feature = "rayon")]
        let slice_iter = slice.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let slice_iter = slice.into_iter();

        slice_iter
    };

    // Filter 1: Filter only cells that have fan out from mapped design cells
    // This is to cut down the number of possible candidates to search
    // let fan_out_from_mapped_design: Option<HashSet<CellWrapper<'d>>> = None;

    // Filter 2: Filter only not already mapped cells
    let not_already_mapped_filter: NotAlreadyMappedConstraint<'p, 'd> =
        NotAlreadyMappedConstraint::new(cell_mapping.clone());

    // Filter 3: If that cell is chosen as a mapping for pattern, it must not invalidate the connectivity specified by by the pattern
    // since cells are chosen in the order inputs -> outputs
    // we check that for each design cell <-> pattern cell, their fan in are connected (since in topological order)
    let connectivity_filter: ConnectivityConstraint<'a, 'p, 'd> = ConnectivityConstraint::new(
        pattern_current.clone(),
        pattern_index,
        design_index,
        pattern,
        design,
        config,
        cell_mapping.clone(),
    );

    candidates
        .filter(|d_candidate| not_already_mapped_filter.d_candidate_is_valid(d_candidate))
        .filter(|d_candidate| connectivity_filter.d_candidate_is_valid(d_candidate))
        .cloned()
        .collect()
}

fn find_isomorphisms_recursive_collect<'a, 'p, 'd>(
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    pattern: &'p Design,
    design: &'d Design,
    config: &'a Config,
    cell_mapping: CellMapping<'p, 'd>,
    mut pattern_mapping_queue: VecDeque<CellWrapper<'p>>,
    depth: usize,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    // Base Case
    let Some(pattern_current) = pattern_mapping_queue.pop_front() else {
        // attach
        return vec![SubgraphIsomorphism {
            mapping: cell_mapping,
            input_by_name: pattern_index.get_input_by_name().clone(),
            output_by_name: pattern_index.get_output_by_name().clone(),
        }];
    };

    let candidates_vec = build_candidates(
        pattern_current.clone(),
        pattern_index,
        design_index,
        pattern,
        design,
        config,
        &cell_mapping,
    );

    #[cfg(feature = "rayon")]
    let cand_iter = candidates_vec.into_par_iter();

    #[cfg(not(feature = "rayon"))]
    let cand_iter = candidates_vec.into_iter();

    let results = cand_iter.map(|d_candidate| {
        trace!(
            "Trying candidate {:?} for pattern cell {:?}",
            d_candidate, pattern_current
        );
        let mut nm = cell_mapping.clone();
        nm.insert(pattern_current.clone(), d_candidate.clone());

        find_isomorphisms_recursive_collect(
            pattern_index,
            design_index,
            pattern,
            design,
            config,
            nm,
            pattern_mapping_queue.clone(),
            depth + 1,
        )
    });

    // let results_debug = results.clone().collect::<Vec<_>>();
    // println!("{:?}", results_debug.len());

    let flat_results: Vec<SubgraphIsomorphism<'p, 'd>> = results.flatten().collect();
    debug!("Depth {} returning {} results", depth, flat_results.len());
    flat_results
}
