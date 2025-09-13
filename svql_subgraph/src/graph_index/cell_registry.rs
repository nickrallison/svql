use crate::cell::{CellIndex, CellKind, CellWrapper};
use prjunnamed_netlist::CellRef;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CellRegistry<'a> {
    /// Nodes in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    /// Maps cell debug index to internal CellIndex
    cell_id_map: HashMap<usize, CellIndex>,
    /// Maps cell types to lists of cells of that type
    cell_type_map: HashMap<CellKind, Vec<CellIndex>>,
}

impl<'a> CellRegistry<'a> {
    pub fn build(cell_refs_topo: &[CellRef<'a>]) -> Self {
        let cells_topo = Self::build_cells_topo(cell_refs_topo);
        let cell_id_map = Self::build_cell_id_map(&cells_topo);
        let cell_type_map = Self::build_cell_type_map(cell_refs_topo);

        CellRegistry {
            cells_topo,
            cell_id_map,
            cell_type_map,
        }
    }

    fn build_cells_topo(cell_refs_topo: &[CellRef<'a>]) -> Vec<CellWrapper<'a>> {
        cell_refs_topo
            .iter()
            .cloned()
            .map(|cell_ref| cell_ref.into())
            .collect()
    }

    fn build_cell_id_map(cells_topo: &[CellWrapper<'a>]) -> HashMap<usize, CellIndex> {
        cells_topo
            .iter()
            .enumerate()
            .map(|(idx, cell)| (cell.debug_index(), CellIndex::new(idx)))
            .collect()
    }

    fn build_cell_type_map(cell_refs_topo: &[CellRef<'a>]) -> HashMap<CellKind, Vec<CellIndex>> {
        let mut cell_type_map: HashMap<CellKind, Vec<CellIndex>> = HashMap::new();
        for (idx, cell) in cell_refs_topo.iter().enumerate() {
            let node_type = CellKind::from(cell.get().as_ref());
            cell_type_map
                .entry(node_type)
                .or_default()
                .push(CellIndex::new(idx));
        }
        cell_type_map
    }

    pub fn cells_of_type(&self, node_type: CellKind) -> Vec<CellWrapper<'a>> {
        self.cell_type_map
            .get(&node_type)
            .map(|v| {
                v.iter()
                    .map(|idx| self.cells_topo[idx.index()].clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    pub fn cells_topo(&self) -> &[CellWrapper<'a>] {
        &self.cells_topo
    }

    pub fn cell_id_map(&self) -> &HashMap<usize, CellIndex> {
        &self.cell_id_map
    }

    pub fn get_cell_by_index(&self, index: CellIndex) -> &CellWrapper<'a> {
        &self.cells_topo[index.index()]
    }

    pub fn get_cell_index(&self, cell: &CellWrapper<'a>) -> Option<CellIndex> {
        self.cell_id_map.get(&cell.debug_index()).copied()
    }
}
