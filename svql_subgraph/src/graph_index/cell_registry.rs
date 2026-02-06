use crate::cell::{CellIndex, CellKind, CellWrapper};
use prjunnamed_netlist::CellRef;
use ahash::AHashMap;

#[derive(Clone, Debug)]
pub struct CellRegistry<'a> {
    /// Nodes in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    /// Maps cell debug index to internal `CellIndex`
    cell_id_map: AHashMap<usize, CellIndex>,
    /// Maps cell types to lists of cell indices
    cell_type_indices: AHashMap<CellKind, Vec<CellIndex>>,
}

impl<'a> CellRegistry<'a> {
    #[must_use] 
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

    #[must_use] 
    pub const fn len(&self) -> usize {
        self.cells_topo.len()
    }

    #[must_use] 
    pub const fn is_empty(&self) -> bool {
        self.cells_topo.is_empty()
    }

    fn build_cells_topo(cell_refs_topo: &[CellRef<'a>]) -> Vec<CellWrapper<'a>> {
        cell_refs_topo
            .iter()
            .copied()
            .map(std::convert::Into::into)
            .collect()
    }

    fn build_cell_id_map(cells_topo: &[CellWrapper<'a>]) -> AHashMap<usize, CellIndex> {
        cells_topo
            .iter()
            .enumerate()
            .map(|(idx, cell)| (cell.debug_index(), CellIndex::new(idx)))
            .collect()
    }

    fn build_cell_type_indices(
        cell_refs_topo: &[CellRef<'a>],
    ) -> AHashMap<CellKind, Vec<CellIndex>> {
        let mut cell_type_indices: AHashMap<CellKind, Vec<CellIndex>> = AHashMap::new();
        for (idx, cell) in cell_refs_topo.iter().enumerate() {
            let node_type = CellKind::from(cell.get().as_ref());
            cell_type_indices
                .entry(node_type)
                .or_default()
                .push(CellIndex::new(idx));
        }
        cell_type_indices
    }

    #[must_use] 
    pub fn cells_of_type_iter(
        &self,
        node_type: CellKind,
    ) -> Option<impl Iterator<Item = &CellWrapper<'a>>> {
        self.cell_type_indices
            .get(&node_type)
            .map(|indices| indices.iter().map(|idx| &self.cells_topo[idx.index()]))
    }

    #[must_use] 
    pub fn cells_of_type_indices(&self, node_type: CellKind) -> &[CellIndex] {
        self.cell_type_indices
            .get(&node_type)
            .map_or(&[], std::vec::Vec::as_slice)
    }

    #[must_use] 
    pub fn cells_topo(&self) -> &[CellWrapper<'a>] {
        &self.cells_topo
    }

    #[must_use] 
    pub const fn cell_id_map(&self) -> &AHashMap<usize, CellIndex> {
        &self.cell_id_map
    }

    #[must_use] 
    pub fn get_cell_by_index(&self, index: CellIndex) -> &CellWrapper<'a> {
        &self.cells_topo[index.index()]
    }

    #[must_use] 
    pub fn get_cell_index(&self, cell: &CellWrapper<'a>) -> Option<CellIndex> {
        self.cell_id_map.get(&cell.debug_index()).copied()
    }

    #[must_use] 
    pub fn indices_to_cells(&self, indices: &[CellIndex]) -> Vec<CellWrapper<'a>> {
        indices
            .iter()
            .map(|idx| self.cells_topo[idx.index()].clone())
            .collect()
    }

    #[must_use] 
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
