use std::collections::HashMap;

use prjunnamed_netlist::Design;

use crate::cell::CellWrapper;

use super::cell::CellKind;
use super::ports::{CellPins, extract_pins};

pub(super) type NodeId = u32;

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

        for cell in design.iter_cells().map(CellWrapper::new) {
            let k = CellKind::from(cell.get().as_ref());
            if !k.is_gate() {
                continue;
            }
            let id = nodes.len() as NodeId;

            nodes.push(cell);
            kinds.push(k);
            let p = extract_pins(cell);
            pins.push(p);
            by_kind.entry(k).or_default().push(id);
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

    // Deterministic iteration over kinds: sort by CellKind (which derives Ord).
    pub(super) fn by_kind_iter(&self) -> Vec<(&CellKind, &[NodeId])> {
        let mut items: Vec<(&CellKind, &Vec<NodeId>)> = self.by_kind.iter().collect();
        items.sort_by_key(|(k, _)| **k);
        items.into_iter().map(|(k, v)| (k, v.as_slice())).collect()
    }
}

#[cfg(test)]
mod tests {

    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn build_index_has_gates() {
        let d = &*SDFFE;
        let idx = Index::build(d);
        assert!(idx.gate_count() > 0);
        assert_eq!(idx.of_kind(super::CellKind::Dff).len() > 0, true);
    }
}
