//! Mapping between needle and haystack cells.

use contracts::*;
use svql_common::*;

use crate::cell::GraphNodeIdx;

/// A collection of mappings found during a search.
#[derive(Clone, Debug, Default)]
pub struct AssignmentSet {
    /// The specific mappings found in the haystack.
    pub items: Vec<SingleAssignment>,
}

impl AssignmentSet {
    /// Creates a set from a list of assignments.
    #[must_use]
    pub const fn new(items: Vec<SingleAssignment>) -> Self {
        Self { items }
    }

    /// Returns true if no matches were found.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the count of found matches.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }
}

/// A partial mapping of cell assignments during the search.
#[derive(Clone, Debug, Default)]
pub struct SingleAssignment {
    /// Pattern to Design cell index mapping
    needle_to_haystack: HashMap<GraphNodeIdx, GraphNodeIdx>,
    /// Design to Pattern cell index mapping
    haystack_to_needle: HashMap<GraphNodeIdx, Vec<GraphNodeIdx>>,
}

impl SingleAssignment {
    /// Checks if the mapping is internally consistent.
    ///
    /// Verifies that the needle-to-haystack and haystack-to-needle maps
    /// maintain a valid bijective relationship.
    pub fn is_consistent(&self) -> bool {
        let total_needle_mappings: usize = self.haystack_to_needle.values().map(|v| v.len()).sum();
        if total_needle_mappings != self.needle_to_haystack.len() {
            return false;
        }
        for (needle, haystack) in &self.needle_to_haystack {
            if !self
                .haystack_to_needle
                .get(haystack)
                .is_some_and(|v| v.contains(needle))
            {
                return false;
            }
        }
        true
    }

    /// Creates a new, empty mapping between needle and haystack.
    #[ensures(ret.is_consistent())]
    pub(super) fn new() -> Self {
        Self {
            needle_to_haystack: HashMap::default(),
            haystack_to_needle: HashMap::default(),
        }
    }

    /// Record a match between a needle node and a haystack node.
    #[ensures(self.is_consistent())]
    pub(super) fn assign(&mut self, needle: GraphNodeIdx, haystack: GraphNodeIdx) {
        self.remove_by_needle(needle);
        self.needle_to_haystack.insert(needle, haystack);
        self.haystack_to_needle
            .entry(haystack)
            .or_default()
            .push(needle);
    }

    /// Removes an assignment by its needle index.
    #[allow(dead_code)]
    #[ensures(self.is_consistent())]
    pub(super) fn remove_by_needle(&mut self, needle: GraphNodeIdx) -> Option<GraphNodeIdx> {
        if let Some(haystack_idx) = self.needle_to_haystack.remove(&needle) {
            if let Some(needles) = self.haystack_to_needle.get_mut(&haystack_idx) {
                needles.retain(|&n| n != needle);
                if needles.is_empty() {
                    self.haystack_to_needle.remove(&haystack_idx);
                }
            }
            return Some(haystack_idx);
        }
        None
    }

    /// Resolves a needle cell index to its matched haystack counterpart.
    #[must_use]
    pub fn get_haystack_cell(&self, needle: GraphNodeIdx) -> Option<GraphNodeIdx> {
        self.needle_to_haystack.get(&needle).copied()
    }

    /// Returns all needle cells that map to a specific haystack cell.
    #[must_use]
    pub fn get_needle_cells(&self, haystack: GraphNodeIdx) -> &[GraphNodeIdx] {
        self.haystack_to_needle
            .get(&haystack)
            .map_or(&[], |v| v.as_slice())
    }

    /// Returns true if the haystack cell is not yet assigned.
    #[must_use]
    pub fn haystack_is_free(&self, haystack: GraphNodeIdx) -> bool {
        !self.haystack_to_needle.contains_key(&haystack)
    }

    /// Returns the number of assigned cells in this mapping.
    #[must_use]
    pub fn len(&self) -> usize {
        self.needle_to_haystack.len()
    }

    /// Returns true if no assignments have been made.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.needle_to_haystack.len(), self.haystack_to_needle.len());
        self.needle_to_haystack.is_empty()
    }

    /// Access the raw haystack-to-needle mapping.
    #[must_use]
    pub const fn haystack_mapping(&self) -> &HashMap<GraphNodeIdx, Vec<GraphNodeIdx>> {
        &self.haystack_to_needle
    }

    /// Access the raw needle-to-haystack mapping.
    #[must_use]
    pub const fn needle_mapping(&self) -> &HashMap<GraphNodeIdx, GraphNodeIdx> {
        &self.needle_to_haystack
    }

    /// Generates a sorted, unique list of haystack indices included in this match.
    pub(super) fn signature(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .needle_to_haystack
            .values()
            .map(|idx| idx.as_usize())
            .collect();
        sig.sort_unstable();
        sig.dedup();
        sig
    }

    /// Generates a bitmask-filtered signature for deduplication.
    pub fn signature_with_mask(&self, mask: &[bool]) -> Vec<u32> {
        self.needle_to_haystack
            .iter()
            .filter(|(n, _)| mask[n.as_usize()])
            .map(|(_, &h)| h.into()) // using From<GraphNodeIdx> for u32
            .collect()
    }

    /// Signature excluding I/O cells — uses pre-computed mask.
    #[must_use]
    pub fn internal_signature(&self, mask: &[bool]) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .needle_mapping()
            .iter()
            .filter(|(needle_idx, _)| mask[needle_idx.as_usize()])
            .map(|(_, haystack_idx)| haystack_idx.as_usize())
            .collect();
        sig.sort_unstable();

        // If there are no internal cells (pure I/O pattern), fall back to full signature
        // to avoid incorrectly deduplicating distinct matches
        if sig.is_empty() {
            return self.signature();
        }

        sig
    }

    /// Generates a signature focusing only on filtered logic gates.
    pub fn filtered_signature(&self, mask: &[bool]) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .needle_to_haystack
            .iter()
            .filter(|(n_idx, _)| {
                // Only include the node if the mask says it's an internal logic gate
                mask.get((n_idx).as_usize()).copied().unwrap_or(false)
            })
            .map(|(_, &h_idx)| h_idx.into())
            .collect();

        sig.sort_unstable();
        sig
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment_consistency() {
        let mut assignment = SingleAssignment::new();
        let n1 = GraphNodeIdx::new(1);
        let h1 = GraphNodeIdx::new(10);

        assignment.assign(n1, h1);
        assert_eq!(assignment.len(), 1);
        assert_eq!(assignment.get_haystack_cell(n1), Some(h1));
        assert_eq!(assignment.get_needle_cells(h1), &[n1]);
    }

    #[test]
    fn test_assignment_multiple_needle_to_same_haystack() {
        let mut assignment = SingleAssignment::new();
        let n1 = GraphNodeIdx::new(1);
        let n2 = GraphNodeIdx::new(2);
        let h1 = GraphNodeIdx::new(10);

        assignment.assign(n1, h1);
        assignment.assign(n2, h1);

        // len() should be the number of needle cells assigned
        assert_eq!(assignment.len(), 2);
        assert_eq!(assignment.get_haystack_cell(n1), Some(h1));
        assert_eq!(assignment.get_haystack_cell(n2), Some(h1));

        let needle_cells = assignment.get_needle_cells(h1);
        assert_eq!(needle_cells.len(), 2);
        assert!(needle_cells.contains(&n1));
        assert!(needle_cells.contains(&n2));
    }

    #[test]
    fn test_assignment_remove() {
        let mut assignment = SingleAssignment::new();
        let n1 = GraphNodeIdx::new(1);
        let h1 = GraphNodeIdx::new(10);

        assignment.assign(n1, h1);
        assert_eq!(assignment.len(), 1);

        assignment.remove_by_needle(n1);
        assert_eq!(assignment.len(), 0);
        assert!(assignment.is_empty());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, quickcheck};

    #[derive(Clone, Debug)]
    struct ArbitraryAssignmentOp {
        needle: GraphNodeIdx,
        haystack: GraphNodeIdx,
        is_remove: bool, // If true, we try to remove; if false, we assign
    }

    impl Arbitrary for ArbitraryAssignmentOp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                needle: GraphNodeIdx::new(u32::arbitrary(g) % 100),
                haystack: GraphNodeIdx::new(u32::arbitrary(g) % 100),
                is_remove: bool::arbitrary(g),
            }
        }
    }

    // Stateful property test: apply a list of ops, check invariant at the end
    quickcheck! {
        fn prop_assignment_ops_maintain_consistency(ops: Vec<ArbitraryAssignmentOp>) -> bool {
            let mut assignment = SingleAssignment::new();
            
            for op in ops {
                if op.is_remove {
                    assignment.remove_by_needle(op.needle);
                } else {
                    assignment.assign(op.needle, op.haystack);
                }
                
                // Check invariant after every step
                if !assignment.is_consistent() {
                    return false;
                }
            }
            true
        }

        fn prop_assignment_bijection(ops: Vec<ArbitraryAssignmentOp>) -> bool {
            let mut assignment = SingleAssignment::new();
            for op in ops {
                if !op.is_remove {
                    assignment.assign(op.needle, op.haystack);
                }
            }

            // Check bijection property
            for (n, h) in assignment.needle_mapping() {
                let needles = assignment.get_needle_cells(*h);
                if !needles.contains(n) {
                    return false;
                }
            }
            true
        }
    }
}
