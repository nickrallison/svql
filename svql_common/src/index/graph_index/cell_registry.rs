//! Registry for mapping physical cells to local graph indices.
//!
//! This module maintains the identity of cells across different query
//! phases and allows for O(1) lookups by cell kind or ID.

use crate::cell::{CellKind, CellWrapper, GraphNodeIdx};
use crate::*;
use prjunnamed_netlist::CellRef;

/// Maintains a mapping of all cells in a design, keyed by their physical and graph IDs.
#[derive(Clone, Debug)]
pub struct CellRegistry<'a> {
    /// Nodes in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    /// Maps cell debug index to internal `GraphNodeIdx`
    cell_id_map: HashMap<usize, GraphNodeIdx>,
    /// Maps cell types to lists of cell indices
    cell_type_indices: HashMap<CellKind, Vec<GraphNodeIdx>>,
}

impl<'a> CellRegistry<'a> {
    /// Builds a registry from a topologically ordered list of cell references.
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

    /// Returns the total number of cells in the registry.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.cells_topo.len()
    }

    /// Returns true if no cells are registered.
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

    fn build_cell_id_map(cells_topo: &[CellWrapper<'a>]) -> HashMap<usize, GraphNodeIdx> {
        cells_topo
            .iter()
            .enumerate()
            .map(|(idx, cell)| {
                (
                    cell.debug_index().storage_key() as usize,
                    GraphNodeIdx::new(idx as u32),
                )
            })
            .collect()
    }

    fn build_cell_type_indices(
        cell_refs_topo: &[CellRef<'a>],
    ) -> HashMap<CellKind, Vec<GraphNodeIdx>> {
        let mut cell_type_indices: HashMap<CellKind, Vec<GraphNodeIdx>> = HashMap::new();
        for (idx, cell) in cell_refs_topo.iter().enumerate() {
            let node_type = CellKind::from(cell.get().as_ref());
            cell_type_indices
                .entry(node_type)
                .or_default()
                .push(GraphNodeIdx::new(idx as u32));
        }
        cell_type_indices
    }

    /// Returns all indices of cells matching the provided type.
    #[must_use]
    pub fn cells_of_type_indices(&self, node_type: CellKind) -> &[GraphNodeIdx] {
        self.cell_type_indices
            .get(&node_type)
            .map_or(&[], std::vec::Vec::as_slice)
    }

    /// Returns the translation map from physical debug IDs to local graph indices.
    #[must_use]
    pub const fn cell_id_map(&self) -> &HashMap<usize, GraphNodeIdx> {
        &self.cell_id_map
    }

    /// Resolves a graph index to its corresponding cell wrapper.
    #[must_use]
    pub fn get_cell_by_index(&self, index: GraphNodeIdx) -> &CellWrapper<'a> {
        &self.cells_topo[index.as_usize()]
    }
}
