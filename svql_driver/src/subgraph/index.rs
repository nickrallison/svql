use std::collections::HashMap;

use prjunnamed_netlist::Design;

use crate::subgraph::cell_kind::CellWrapper;

use super::cell_kind::CellKind;
use super::ports::{CellPins, extract_pins};

pub(super) type NodeId = u32;

#[derive(Clone, Debug)]
pub(super) struct Index<'a> {
    design: &'a Design,
    nodes: Vec<CellWrapper<'a>>,
    kinds: Vec<CellKind>,
    pins: Vec<CellPins<'a>>,
    by_kind: HashMap<CellKind, Vec<NodeId>>,
    cell_to_id: HashMap<CellWrapper<'a>, NodeId>,
    gate_count: usize,
}

impl<'a> Index<'a> {
    pub(super) fn build(design: &'a Design) -> Self {
        let mut nodes: Vec<CellWrapper<'a>> = Vec::new();
        let mut kinds: Vec<CellKind> = Vec::new();
        let mut pins: Vec<CellPins<'a>> = Vec::new();
        let mut by_kind: HashMap<CellKind, Vec<NodeId>> = HashMap::new();
        let mut cell_to_id: HashMap<CellWrapper<'a>, NodeId> = HashMap::new();
        let mut gate_count = 0usize;

        for cell in design.iter_cells().map(|c| CellWrapper::new(c)) {
            let k = CellKind::from(cell.get().as_ref());
            if !(k.is_gate()) {
                continue;
            }
            let id = nodes.len() as NodeId;
            gate_count += 1;
            nodes.push(cell);
            kinds.push(k);
            let p = extract_pins(cell);
            pins.push(p);
            by_kind.entry(k).or_default().push(id);
            cell_to_id.insert(cell, id);
        }

        Index {
            design,
            nodes,
            kinds,
            pins,
            by_kind,
            cell_to_id,
            gate_count,
        }
    }

    pub(super) fn design(&self) -> &'a Design {
        self.design
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
        self.gate_count
    }

    pub(super) fn try_cell_to_node(&self, c: CellWrapper<'a>) -> Option<NodeId> {
        self.cell_to_id.get(&c).copied()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Driver;
    use crate::util::load_driver_from;

    lazy_static::lazy_static! {
        static ref SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/sdffe.v").unwrap();
    }

    #[test]
    fn build_index_has_gates() {
        let d = &*SDFFE;
        let idx = Index::build(d.design_as_ref());
        assert!(idx.gate_count() > 0);
        assert_eq!(idx.of_kind(CellKind::Dff).len() > 0, true);
        assert_eq!(idx.of_kind(CellKind::And).len() >= 0, true);
    }
}
