//! Mapping between needle and haystack cells.

use std::collections::HashMap;

use crate::cell::{CellKind, CellWrapper};

/// A collection of mappings found during a search.
#[derive(Clone, Debug, Default)]
pub struct AssignmentSet<'needle, 'haystack> {
    pub items: Vec<SingleAssignment<'needle, 'haystack>>,
    pub needle_input_fanout_by_name: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
    pub needle_output_fanin_by_name: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
}

impl<'needle, 'haystack> AssignmentSet<'needle, 'haystack> {
    #[must_use]
    pub const fn new(
        items: Vec<SingleAssignment<'needle, 'haystack>>,
        needle_input_fanout_by_name: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
        needle_output_fanin_by_name: HashMap<String, Vec<(CellWrapper<'needle>, usize)>>,
    ) -> Self {
        Self {
            items,
            needle_input_fanout_by_name,
            needle_output_fanin_by_name,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }
}

/// A partial mapping of cell assignments during the search.
#[derive(Clone, Debug, Default)]
pub struct SingleAssignment<'needle, 'haystack> {
    /// Pattern to Design cell mapping
    needle_to_haystack: HashMap<CellWrapper<'needle>, CellWrapper<'haystack>>,
    /// Design to Pattern cell mapping
    haystack_to_needle: HashMap<CellWrapper<'haystack>, CellWrapper<'needle>>,
}

impl<'needle, 'haystack> SingleAssignment<'needle, 'haystack> {
    pub(super) fn new() -> Self {
        Self {
            needle_to_haystack: HashMap::new(),
            haystack_to_needle: HashMap::new(),
        }
    }

    pub(super) fn assign(
        &mut self,
        needle: CellWrapper<'needle>,
        haystack: CellWrapper<'haystack>,
    ) {
        self.needle_to_haystack
            .insert(needle.clone(), haystack.clone());
        self.haystack_to_needle.insert(haystack, needle);
    }

    #[allow(dead_code)]
    pub(super) fn remove_by_needle(
        &mut self,
        needle: CellWrapper<'needle>,
    ) -> Option<CellWrapper<'haystack>> {
        if let Some(haystack_cell) = self.needle_to_haystack.remove(&needle) {
            self.haystack_to_needle.remove(&haystack_cell);
            return Some(haystack_cell);
        }
        None
    }

    #[must_use]
    pub fn get_haystack_cell(
        &self,
        needle: &CellWrapper<'needle>,
    ) -> Option<&CellWrapper<'haystack>> {
        self.needle_to_haystack.get(needle)
    }

    #[must_use]
    pub fn get_needle_cell(
        &self,
        haystack: &CellWrapper<'haystack>,
    ) -> Option<&CellWrapper<'needle>> {
        self.haystack_to_needle.get(haystack)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.needle_to_haystack.len(), self.haystack_to_needle.len());
        self.needle_to_haystack.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.needle_to_haystack.len(), self.haystack_to_needle.len());
        self.needle_to_haystack.is_empty()
    }

    #[allow(clippy::mutable_key_type)]
    #[must_use]
    pub const fn haystack_mapping(&self) -> &HashMap<CellWrapper<'haystack>, CellWrapper<'needle>> {
        &self.haystack_to_needle
    }

    #[allow(clippy::mutable_key_type)]
    #[must_use]
    pub const fn needle_mapping(&self) -> &HashMap<CellWrapper<'needle>, CellWrapper<'haystack>> {
        &self.needle_to_haystack
    }

    pub(super) fn signature(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .needle_to_haystack
            .values()
            .map(super::cell::CellWrapper::debug_index)
            .collect();
        sig.sort_unstable();
        sig.dedup();
        sig
    }

    #[must_use]
    pub fn internal_signature(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .needle_mapping()
            .iter()
            .filter(|(p, _)| !matches!(p.cell_type(), CellKind::Input | CellKind::Output))
            .map(|(_, d)| d.debug_index())
            .collect();
        sig.sort_unstable();
        sig
    }
}
