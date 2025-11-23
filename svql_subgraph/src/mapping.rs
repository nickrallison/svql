//! Bi-directional mapping between needle and haystack cells.

use std::collections::HashMap;

use crate::cell::CellWrapper;

/// Maintains the current state of cell assignments during the search.
#[derive(Clone, Debug, Default)]
pub struct Assignment<'needle, 'haystack> {
    /// Pattern to Design cell mapping
    needle_to_haystack: HashMap<CellWrapper<'needle>, CellWrapper<'haystack>>,
    /// Design to Pattern cell mapping
    haystack_to_needle: HashMap<CellWrapper<'haystack>, CellWrapper<'needle>>,
}

impl<'needle, 'haystack> Assignment<'needle, 'haystack> {
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

    // debug ensure that needle & haystack mappings are consistent
    pub fn get_haystack_cell(
        &self,
        needle: CellWrapper<'needle>,
    ) -> Option<CellWrapper<'haystack>> {
        self.needle_to_haystack.get(&needle).cloned()
    }

    // debug ensure that needle & haystack mappings are consistent
    pub fn get_needle_cell(
        &self,
        haystack: CellWrapper<'haystack>,
    ) -> Option<CellWrapper<'needle>> {
        self.haystack_to_needle.get(&haystack).cloned()
    }

    pub fn len(&self) -> usize {
        debug_assert_eq!(self.needle_to_haystack.len(), self.haystack_to_needle.len());
        self.needle_to_haystack.len()
    }

    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.needle_to_haystack.len(), self.haystack_to_needle.len());
        self.needle_to_haystack.is_empty()
    }

    pub fn haystack_mapping(&self) -> &HashMap<CellWrapper<'haystack>, CellWrapper<'needle>> {
        &self.haystack_to_needle
    }

    pub fn needle_mapping(&self) -> &HashMap<CellWrapper<'needle>, CellWrapper<'haystack>> {
        &self.needle_to_haystack
    }

    pub(super) fn signature(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .needle_to_haystack
            .values()
            .map(|d| d.debug_index())
            .collect();
        sig.sort_unstable();
        sig.dedup();
        sig
    }
}
