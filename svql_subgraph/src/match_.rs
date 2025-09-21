use std::collections::{HashMap, VecDeque};

use crate::mapping::Assignment;
use svql_design_set::cell::{CellKind, CellWrapper};

#[derive(Clone, Debug)]
pub struct Matches<'needle, 'haystack> {
    pub items: Vec<Match<'needle, 'haystack>>,
    pub needle_input_fanout_by_name: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
    pub needle_output_fanin_by_name: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
}

#[derive(Clone, Debug, Default)]
pub struct Match<'needle, 'haystack> {
    pub assignment: Assignment<'needle, 'haystack>,
}

impl<'needle, 'haystack> Match<'needle, 'haystack> {
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

pub(crate) struct PartialMatch<'needle, 'haystack> {
    pub(crate) assignment: Assignment<'needle, 'haystack>,
    pub(crate) needle_input_mapping_queue: VecDeque<CellWrapper<'needle>>,
    pub(crate) needle_internal_mapping_queue: VecDeque<CellWrapper<'needle>>,
}

impl<'needle, 'haystack> PartialMatch<'needle, 'haystack> {
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
}

impl<'needle, 'haystack> TryFrom<PartialMatch<'needle, 'haystack>> for Match<'needle, 'haystack> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: PartialMatch<'needle, 'haystack>) -> Result<Self, Self::Error> {
        if !value.needle_input_mapping_queue.is_empty()
            || !value.needle_internal_mapping_queue.is_empty()
        {
            return Err(format!(
                "Cannot convert PartialMatch to Match: mapping queues are not empty"
            )
            .into());
        }
        Ok(Match {
            assignment: value.assignment,
        })
    }
}
