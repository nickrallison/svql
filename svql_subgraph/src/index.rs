use std::{cell, collections::HashMap};

use prjunnamed_netlist::Design;

use crate::model::{CellKind, CellWrapper, Source};

pub(super) type NodeId = u32;

#[derive(Clone, Debug)]
pub(super) struct Index<'a> {
    /// Cells of design in topological order
    cells_topo: Vec<CellWrapper<'a>>,
    by_kind: HashMap<CellKind, Vec<NodeId>>,
    cell_to_id: HashMap<usize, NodeId>,
}

impl<'a> Index<'a> {
    #[contracts::debug_ensures(ret.gate_count() <= design.iter_cells().count())]
    pub(super) fn build(design: &'a Design) -> Self {
        let mut by_kind: HashMap<CellKind, Vec<NodeId>> = HashMap::new();
        let mut cell_to_id: HashMap<usize, NodeId> = HashMap::new();
        // Filter to gates first, then enumerate so NodeIds are contiguous and stable.
        let cells_topo: Vec<CellWrapper<'a>> = design
            .iter_cells_topo()
            .rev()
            .map(CellWrapper::new)
            .filter(|cell| !matches!(cell.kind, CellKind::Name))
            .collect();
        let gates: Vec<CellWrapper<'a>> = cells_topo
            .iter()
            .filter(|cell| cell.kind.is_gate())
            .cloned()
            .collect();

        for (id, cell) in gates.into_iter().enumerate() {
            let id = id as NodeId;
            by_kind.entry(cell.kind).or_default().push(id);
            cell_to_id.insert(cell.debug_index(), id);
        }

        Index {
            cells_topo,
            by_kind,
            cell_to_id,
        }
    }

    pub(super) fn get_node_id(&self, index: usize) -> Option<NodeId> {
        self.cell_to_id.get(&index).copied()
    }

    #[contracts::debug_requires((id as usize) < self.cells_topo.len())]
    pub(super) fn node_to_cell(&self, id: NodeId) -> &CellWrapper<'a> {
        &self.cells_topo[id as usize]
    }

    #[contracts::debug_requires((id as usize) < self.cells_topo.len())]
    pub(super) fn kind(&self, id: NodeId) -> CellKind {
        self.cells_topo[id as usize].kind
    }

    #[contracts::debug_requires((id as usize) < self.cells_topo.len())]
    pub(super) fn pins(&self, id: NodeId) -> &Vec<Source<'a>> {
        &self.cells_topo[id as usize].pins
    }

    pub(super) fn of_kind(&self, k: CellKind) -> &[NodeId] {
        self.by_kind.get(&k).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub(super) fn gate_count(&self) -> usize {
        self.cells_topo.iter().filter(|c| c.kind.is_gate()).count()
    }

    pub(super) fn nodes(&self) -> &[CellWrapper<'a>] {
        &self.cells_topo
    }

    pub(super) fn try_cell_to_node(&self, c: CellWrapper<'a>) -> Option<NodeId> {
        self.cell_to_id.get(&c.debug_index()).copied()
    }

    /// Deterministic owned iteration over kinds.
    pub(super) fn by_kind_iter(&self) -> Vec<(CellKind, Vec<NodeId>)> {
        let mut items: Vec<(CellKind, Vec<NodeId>)> =
            self.by_kind.iter().map(|(k, v)| (*k, v.clone())).collect();
        items.sort_by_key(|kn| kn.0);
        items
    }
    pub(super) fn get_cells_topo(&self) -> Vec<&CellWrapper<'a>> {
        self.cells_topo
            .iter()
            // .filter(|c| c.kind.is_gate() | c.kind.is_input())
            .collect()
    }
}
