use crate::cell::{CellIndex, CellKind, CellWrapper};
use prjunnamed_netlist::CellRef;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CellRegistry<'a> {
    /// Nodes in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    /// Maps cell debug index to internal CellIndex
    cell_id_map: HashMap<usize, CellIndex>,
    /// Maps cell types to lists of cell indices
    cell_type_indices: HashMap<CellKind, Vec<CellIndex>>,
}

impl<'a> CellRegistry<'a> {
    pub fn build(cell_refs_topo: &[CellRef<'a>]) -> Self {
        let cells_topo = Self::build_cells_topo(cell_refs_topo);
        let cell_id_map = Self::build_cell_id_map(&cells_topo);
        let cell_type_indices = Self::build_cell_type_indices(cell_refs_topo);

        CellRegistry {
            cells_topo,
            cell_id_map,
            cell_type_indices,
        }
    }

    pub fn len(&self) -> usize {
        self.cells_topo.len()
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

    fn build_cell_type_indices(
        cell_refs_topo: &[CellRef<'a>],
    ) -> HashMap<CellKind, Vec<CellIndex>> {
        let mut cell_type_indices: HashMap<CellKind, Vec<CellIndex>> = HashMap::new();
        for (idx, cell) in cell_refs_topo.iter().enumerate() {
            let node_type = CellKind::from(cell.get().as_ref());
            cell_type_indices
                .entry(node_type)
                .or_default()
                .push(CellIndex::new(idx));
        }
        cell_type_indices
    }

    pub fn cells_of_type_iter(
        &self,
        node_type: CellKind,
    ) -> Option<impl Iterator<Item = &CellWrapper<'a>>> {
        self.cell_type_indices
            .get(&node_type)
            .map(|indices| indices.iter().map(|idx| &self.cells_topo[idx.index()]))
    }

    pub fn cells_of_type_indices(&self, node_type: CellKind) -> &[CellIndex] {
        self.cell_type_indices
            .get(&node_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
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

    pub fn indices_to_cells(&self, indices: &[CellIndex]) -> Vec<CellWrapper<'a>> {
        indices
            .iter()
            .map(|idx| self.cells_topo[idx.index()].clone())
            .collect()
    }

    pub fn indices_with_ports_to_cells(
        &self,
        indices_with_ports: &[(CellIndex, usize)],
    ) -> Vec<(CellWrapper<'a>, usize)> {
        indices_with_ports
            .iter()
            .map(|(idx, port)| (self.cells_topo[idx.index()].clone(), *port))
            .collect()
    }
}
