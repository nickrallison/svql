//! Mapping between needle and haystack cells.

use svql_common::*;

use crate::cell::CellId;

/// A collection of mappings found during a search.
#[derive(Clone, Debug, Default)]
pub struct AssignmentSet {
    pub items: Vec<SingleAssignment>,
}

impl AssignmentSet {
    #[must_use]
    pub const fn new(items: Vec<SingleAssignment>) -> Self {
        Self { items }
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
pub struct SingleAssignment {
    /// Pattern to Design cell index mapping
    needle_to_haystack: HashMap<CellId, CellId>,
    /// Design to Pattern cell index mapping
    haystack_to_needle: HashMap<CellId, Vec<CellId>>,
}

impl SingleAssignment {
    pub(super) fn new() -> Self {
        Self {
            needle_to_haystack: HashMap::new(),
            haystack_to_needle: HashMap::new(),
        }
    }

    pub(super) fn assign(&mut self, needle: CellId, haystack: CellId) {
        self.needle_to_haystack.insert(needle, haystack);
        self.haystack_to_needle
            .entry(haystack)
            .or_default()
            .push(needle);
    }

    #[allow(dead_code)]
    pub(super) fn remove_by_needle(&mut self, needle: CellId) -> Option<CellId> {
        if let Some(haystack_idx) = self.needle_to_haystack.remove(&needle) {
            self.haystack_to_needle.remove(&haystack_idx);
            return Some(haystack_idx);
        }
        None
    }

    #[must_use]
    pub fn get_haystack_cell(&self, needle: CellId) -> Option<CellId> {
        self.needle_to_haystack.get(&needle).copied()
    }

    #[must_use]
    pub fn get_needle_cells(&self, haystack: CellId) -> &[CellId] {
        self.haystack_to_needle
            .get(&haystack)
            .map_or(&[], |v| v.as_slice())
    }

    /// Returns true if the haystack cell is not yet assigned.
    #[must_use]
    pub fn haystack_is_free(&self, haystack: CellId) -> bool {
        !self.haystack_to_needle.contains_key(&haystack)
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

    #[must_use]
    pub const fn haystack_mapping(&self) -> &HashMap<CellId, Vec<CellId>> {
        &self.haystack_to_needle
    }

    #[must_use]
    pub const fn needle_mapping(&self) -> &HashMap<CellId, CellId> {
        &self.needle_to_haystack
    }

    pub(super) fn signature(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .needle_to_haystack
            .values()
            .map(|idx| idx.index())
            .collect();
        sig.sort_unstable();
        sig.dedup();
        sig
    }

    /// Signature excluding I/O cells — requires the needle graph index to classify cells.
    #[must_use]
    pub fn internal_signature(
        &self,
        needle_index: &crate::graph_index::GraphIndex<'_>,
    ) -> Vec<usize> {
        use crate::cell::CellKind;
        let mut sig: Vec<usize> = self
            .needle_mapping()
            .iter()
            .filter(|(needle_idx, _)| {
                let kind = needle_index.get_cell_by_index(**needle_idx).cell_type();
                !matches!(kind, CellKind::Input | CellKind::Output)
            })
            .map(|(_, haystack_idx)| haystack_idx.index())
            .collect();
        sig.sort_unstable();

        // If there are no internal cells (pure I/O pattern), fall back to full signature
        // to avoid incorrectly deduplicating distinct matches
        if sig.is_empty() {
            return self.signature();
        }

        sig
    }
}
