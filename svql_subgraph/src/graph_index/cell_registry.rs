use crate::cell::{CellIndex, CellKind, CellWrapper};
use prjunnamed_netlist::CellRef;
use svql_common::prelude::*;

#[derive(Clone, Debug)]
pub struct CellRegistry<'a> {
    /// Nodes in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    /// Maps cell debug index to internal `CellIndex`
    cell_id_map: HashMap<usize, CellIndex>,
    /// Maps cell types to lists of cell indices
    cell_type_indices: HashMap<CellKind, Vec<CellIndex>>,
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

    #[must_use]
    pub fn cells_of_type_indices(&self, node_type: CellKind) -> &[CellIndex] {
        self.cell_type_indices
            .get(&node_type)
            .map_or(&[], std::vec::Vec::as_slice)
    }

    #[must_use]
    pub const fn cell_id_map(&self) -> &HashMap<usize, CellIndex> {
        &self.cell_id_map
    }

    #[must_use]
    pub fn get_cell_by_index(&self, index: CellIndex) -> &CellWrapper<'a> {
        &self.cells_topo[index.index()]
    }
}
