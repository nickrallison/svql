use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use log::trace;
use prjunnamed_netlist::{CellHash, Design};

use crate::model::{CellKind, CellPins, CellWrapper, extract_pins};

pub(super) type NodeId = u32;

/// Self-documenting wrapper for "all nodes of a given CellKind".
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct KindNodes {
    pub kind: CellKind,
    pub nodes: Vec<NodeId>,
}

#[derive(Clone, Debug)]
pub(super) struct Index<'a> {
    nodes: Vec<CellWrapper>,
    kinds: Vec<CellKind>,
    pins: Vec<CellPins>,
    by_kind: HashMap<CellKind, Vec<NodeId>>,
    // Key by debug_index to avoid interior mutability key lint.
    cell_to_id: HashMap<usize, NodeId>,
    design: &'a Design,
    design_hash: u64,
}

impl<'a> Index<'a> {
    pub fn build(design: &'a Design) -> Self {
        trace!(
            "Building index for design with {} cells",
            design.iter_cells().count()
        );

        let mut nodes: Vec<CellWrapper> = Vec::new();
        let mut kinds: Vec<CellKind> = Vec::new();
        let mut pins: Vec<CellPins> = Vec::new();
        let mut by_kind: HashMap<CellKind, Vec<NodeId>> = HashMap::new();
        let mut cell_to_id: HashMap<usize, NodeId> = HashMap::new();
        let design_hash = calculate_design_hash(design);

        // Filter to gates first, then enumerate so NodeIds are contiguous and stable.
        let gate_wrappers: Vec<(CellWrapper, CellKind)> = design
            .iter_cells()
            .map(|cref| {
                let cell_hash = CellHash::new(cref.debug_index(), design_hash);
                let cell_wrapper = CellWrapper::new(cell_hash);
                cell_wrapper
            })
            .filter_map(|cell_wrapper| {
                // Convert to ValidCellWrapper to inspect the cell
                let valid_cell = cell_wrapper.into_valid_cell_wrapper_unchecked(design);
                let kind = CellKind::from(valid_cell.get().as_ref());
                if kind.is_gate() {
                    trace!("Found gate: {:?} at index {}", kind, cell_wrapper.index());
                    Some((cell_wrapper, kind))
                } else {
                    trace!(
                        "Skipping non-gate: {:?} at index {}",
                        kind,
                        cell_wrapper.index()
                    );
                    None
                }
            })
            .collect();
        trace!("Found {} gate wrappers", gate_wrappers.len());

        for (id, (cell_wrapper, kind)) in gate_wrappers.into_iter().enumerate() {
            let id = id as NodeId;
            let cell_pins = extract_pins(cell_wrapper, design, design_hash);

            nodes.push(cell_wrapper);
            kinds.push(kind);
            pins.push(cell_pins);

            by_kind.entry(kind).or_default().push(id);
            cell_to_id.insert(cell_wrapper.index(), id);
        }

        trace!("Index built with {} nodes", nodes.len());
        Index {
            nodes,
            kinds,
            pins,
            by_kind,
            cell_to_id,
            design,
            design_hash,
        }
    }

    pub(super) fn node_to_cell(&self, id: NodeId) -> CellWrapper {
        self.nodes[id as usize]
    }

    pub(super) fn kind(&self, id: NodeId) -> CellKind {
        self.kinds[id as usize]
    }

    pub(super) fn pins(&self, id: NodeId) -> &CellPins {
        &self.pins[id as usize]
    }

    pub(super) fn of_kind(&self, k: CellKind) -> &[NodeId] {
        self.by_kind.get(&k).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub(super) fn gate_count(&self) -> usize {
        self.nodes.len()
    }

    pub(super) fn try_cell_to_node(&self, c: CellWrapper) -> Option<NodeId> {
        self.cell_to_id.get(&c.index()).copied()
    }

    /// Deterministic owned iteration over kinds.
    pub(super) fn by_kind_iter(&self) -> Vec<KindNodes> {
        let mut items: Vec<KindNodes> = self
            .by_kind
            .iter()
            .map(|(k, v)| KindNodes {
                kind: *k,
                nodes: v.clone(),
            })
            .collect();
        items.sort_by_key(|kn| kn.kind);
        items
    }

    pub(super) fn design(&self) -> &Design {
        self.design
    }
    pub fn design_hash(&self) -> u64 {
        self.design_hash
    }
}

#[cfg(test)]
mod tests {

    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::test_support::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn build_index_has_gates() {
        let d = &*SDFFE;
        let idx = Index::build(d);
        assert!(idx.gate_count() > 0);
        assert_eq!(idx.of_kind(crate::model::CellKind::Dff).len() > 0, true);
    }

    #[test]
    fn by_kind_iter_is_owned_and_sorted() {
        let d = &*SDFFE;
        let idx = Index::build(d);
        let pairs = idx.by_kind_iter();
        assert!(!pairs.is_empty());
        let mut sorted = pairs.clone();
        sorted.sort_by_key(|kn| kn.kind);
        assert_eq!(pairs, sorted);
    }
}
