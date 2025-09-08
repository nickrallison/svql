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
    pub mapping: CellMapping<'p, 'd>,

    // Boundary IO lookup tables
    pub input_by_name: HashMap<&'p str, CellWrapper<'p>>,
    pub output_by_name: HashMap<&'p str, CellWrapper<'p>>,
    //
    pub bound_inputs: HashMap<&'p str, CellWrapper<'d>>,
    pub bound_outputs: HashMap<String, Vec<(CellWrapper<'d>, usize)>>,
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
    fn input_bindings(&mut self) {
        // self.input_by_name = HashMap::new();
        for (pat_cell, des_cell) in self.mapping.pattern_mapping().clone() {
            if matches!(pat_cell.cell_type(), CellType::Input) {
                let name = pat_cell.input_name().expect("Input must have name");
                let returned = self
                    .mapping
                    .remove_by_pattern(pat_cell)
                    .expect("Input must be mapped");
                if returned != des_cell {
                    panic!("Input mapping inconsistent");
                }
                self.bound_inputs.insert(name, returned);
            }
        }
    }
    fn output_bindings(&mut self, design: &'d Design) {
        let mut bound_outputs: HashMap<String, Vec<(CellWrapper<'d>, usize)>> = HashMap::new();
        for des_cell in design.iter_cells() {
            let cell_wrapper: CellWrapper<'d> = des_cell.into();

            match cell_wrapper.get() {
                prjunnamed_netlist::Cell::Output(name, value) => {
                    let value_repr = &value.0;
                    let mut visited: Vec<(CellWrapper<'d>, usize)> = Vec::new();

                    match value_repr {
                        prjunnamed_netlist::ValueRepr::None => {}
                        prjunnamed_netlist::ValueRepr::Some(net) => {
                            let cell_res = design.find_cell(*net);
                            if let Ok((cell_ref, id)) = cell_res {
                                visited.push((cell_ref.into(), id));
                            }
                        }
                        prjunnamed_netlist::ValueRepr::Many(nets) => {
                            for net in nets {
                                let cell_res = design.find_cell(*net);
                                if let Ok((cell_ref, id)) = cell_res {
                                    visited.push((cell_ref.into(), id));
                                }
                            }
                        }
                    }
                    bound_outputs.insert(name.to_string(), visited);
                }
                _ => {}
            }
        }
        self.bound_outputs = bound_outputs
    }
}

pub fn find_subgraph_isomorphisms<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    let pattern_index = DesignIndex::build(pattern);
    let design_index = DesignIndex::build(design);

    find_subgraph_isomorphisms_index(pattern, design, &pattern_index, &design_index, config)
}

pub fn find_subgraph_isomorphisms_index<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    pattern_index: &DesignIndex<'p>,
    design_index: &DesignIndex<'d>,
    config: &Config,
) -> Vec<SubgraphIsomorphism<'p, 'd>> {
    info!("Starting subgraph isomorphism search");
    trace!("Config: {:?}", config);

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

    // Filter results out if they have inconsistent input bindings
    // if config.bind_inputs {
    //     results.retain(|m| {});
    // }

    if config.dedupe {
        let mut seen: HashSet<Vec<usize>> = HashSet::new();
        results.retain(|m| seen.insert(m.mapping.signature()));
        info!("After AutoMorph deduplication: {} results", results.len());
    }

    info!("Final result count: {}", results.len());

    results
}

fn build_pattern_mapping_queue<'p>(pattern_index: &DesignIndex<'p>) -> VecDeque<CellWrapper<'p>> {
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
    pattern_index: &'a DesignIndex<'p>,
    design_index: &'a DesignIndex<'d>,
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

fn find_isomorphisms_recursive_inputs_collect<'a, 'p, 'd>(
    pattern_index: &'a DesignIndex<'p>,
    design_index: &'a DesignIndex<'d>,
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
        let mut mapping = SubgraphIsomorphism {
            mapping: cell_mapping,
            input_by_name: pattern_index.get_input_by_name().clone(),
            output_by_name: pattern_index.get_output_by_name().clone(),
            bound_inputs: HashMap::new(),
            bound_outputs: HashMap::new(),
        };

        mapping.input_bindings();
        mapping.output_bindings(design);

        return vec![mapping];
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

fn find_isomorphisms_recursive_collect<'a, 'p, 'd>(
    pattern_index: &'a DesignIndex<'p>,
    design_index: &'a DesignIndex<'d>,
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
        let mut mapping = SubgraphIsomorphism {
            mapping: cell_mapping,
            input_by_name: pattern_index.get_input_by_name().clone(),
            output_by_name: pattern_index.get_output_by_name().clone(),
            bound_inputs: HashMap::new(),
            bound_outputs: HashMap::new(),
        };

        mapping.input_bindings();
        mapping.output_bindings(design);

        return vec![mapping];
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
