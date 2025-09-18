use std::collections::HashMap;

use crate::mapping::Assignment;
use svql_design_set::cell::{CellKind, CellWrapper};

#[derive(Clone, Debug)]
pub struct EmbeddingSet<'needle, 'haystack> {
    pub items: Vec<Embedding<'needle, 'haystack>>,
    pub needle_input_fanout_by_name: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
    pub needle_output_fanin_by_name: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
}

#[derive(Clone, Debug, Default)]
pub struct Embedding<'needle, 'haystack> {
    pub assignment: Assignment<'needle, 'haystack>,
}

impl<'needle, 'haystack> Embedding<'needle, 'haystack> {
    pub fn len(&self) -> usize {
        self.assignment.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assignment.is_empty()
    }

    pub fn debug_print(&self) {
        let mapping = self.assignment.needle_mapping();
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
            .assignment
            .needle_mapping()
            .iter()
            .filter(|(p, _)| !matches!(p.cell_type(), CellKind::Input | CellKind::Output))
            .map(|(_, d)| d.debug_index())
            .collect();
        sig.sort_unstable();
        sig
    }
}
