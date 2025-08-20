use std::collections::HashMap;

use prjunnamed_netlist::Design;

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
    nodes: Vec<CellWrapper<'a>>,
    kinds: Vec<CellKind>,
    pins: Vec<CellPins<'a>>,
    by_kind: HashMap<CellKind, Vec<NodeId>>,
    // Key by debug_index to avoid interior mutability key lint.
    cell_to_id: HashMap<usize, NodeId>,
}

impl<'a> Index<'a> {
    pub(super) fn build(design: &'a Design) -> Self {
        let mut nodes: Vec<CellWrapper<'a>> = Vec::new();
        let mut kinds: Vec<CellKind> = Vec::new();
        let mut pins: Vec<CellPins<'a>> = Vec::new();
        let mut by_kind: HashMap<CellKind, Vec<NodeId>> = HashMap::new();
        let mut cell_to_id: HashMap<usize, NodeId> = HashMap::new();

        // Filter to gates first, then enumerate so NodeIds are contiguous and stable.
        let gate_triplets: Vec<(CellWrapper<'a>, CellKind, CellPins<'a>)> = design
            .iter_cells()
            .map(CellWrapper::new)
            .filter_map(|cell| {
                let kind = CellKind::from(cell.get().as_ref());
                kind.is_gate().then_some((cell, kind, extract_pins(cell)))
            })
            .collect();

        for (id, (cell, kind, cell_pins)) in gate_triplets.into_iter().enumerate() {
            let id = id as NodeId;
            nodes.push(cell);
            kinds.push(kind);
            pins.push(cell_pins);

            by_kind.entry(kind).or_default().push(id);
            cell_to_id.insert(cell.debug_index(), id);
        }

        Index {
            nodes,
            kinds,
            pins,
            by_kind,
            cell_to_id,
        }
    }

    pub(super) fn node_to_cell(&self, id: NodeId) -> CellWrapper<'a> {
        self.nodes[id as usize]
    }

    pub(super) fn kind(&self, id: NodeId) -> CellKind {
        self.kinds[id as usize]
    }

    pub(super) fn pins(&self, id: NodeId) -> &CellPins<'a> {
        &self.pins[id as usize]
    }

    pub(super) fn of_kind(&self, k: CellKind) -> &[NodeId] {
        self.by_kind.get(&k).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub(super) fn gate_count(&self) -> usize {
        self.nodes.len()
    }

    pub(super) fn try_cell_to_node(&self, c: CellWrapper<'a>) -> Option<NodeId> {
        self.cell_to_id.get(&c.debug_index()).copied()
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
