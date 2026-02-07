use crate::cell::{CellId, CellKind, CellWrapper};
use prjunnamed_netlist::CellRef;
use svql_common::*;

#[derive(Clone, Debug)]
pub struct CellRegistry<'a> {
    /// Nodes in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    /// Maps cell debug index to internal `CellId`
    cell_id_map: HashMap<usize, CellId>,
    /// Maps cell types to lists of cell indices
    cell_type_indices: HashMap<CellKind, Vec<CellId>>,
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

    fn build_cell_id_map(cells_topo: &[CellWrapper<'a>]) -> HashMap<usize, CellId> {
        cells_topo
            .iter()
            .enumerate()
            .map(|(idx, cell)| (cell.debug_index(), CellId::new(idx)))
            .collect()
    }

    fn build_cell_type_indices(
        cell_refs_topo: &[CellRef<'a>],
    ) -> HashMap<CellKind, Vec<CellId>> {
        let mut cell_type_indices: HashMap<CellKind, Vec<CellId>> = HashMap::new();
        for (idx, cell) in cell_refs_topo.iter().enumerate() {
            let node_type = CellKind::from(cell.get().as_ref());
            cell_type_indices
                .entry(node_type)
                .or_default()
                .push(CellId::new(idx));
        }
        cell_type_indices
    }

    #[must_use]
    pub fn cells_of_type_indices(&self, node_type: CellKind) -> &[CellId] {
        self.cell_type_indices
            .get(&node_type)
            .map_or(&[], std::vec::Vec::as_slice)
    }

    #[must_use]
    pub const fn cell_id_map(&self) -> &HashMap<usize, CellId> {
        &self.cell_id_map
    }

    #[must_use]
    pub fn get_cell_by_index(&self, index: CellId) -> &CellWrapper<'a> {
        &self.cells_topo[index.index()]
    }
}
