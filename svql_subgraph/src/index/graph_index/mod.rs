//! Graph indexing and connectivity analysis.
//!
//! Provides efficient data structures for querying graph connectivity,
//! cell types, and I/O mappings.

mod cell_registry;
mod connectivity_graph;
mod io_mapping;

pub use cell_registry::CellRegistry;
pub use connectivity_graph::ConnectivityGraph;
pub use io_mapping::IoMapping;

use crate::cell::{CellId, CellKind, CellWrapper};
use prjunnamed_netlist::{CellRef, Design};
use svql_common::*;

/// An index over a design graph providing fast access to connectivity and cell data.
#[derive(Clone, Debug)]
pub struct GraphIndex<'a> {
    cell_registry: CellRegistry<'a>,
    connectivity: ConnectivityGraph,
    io_mapping: IoMapping,
}

impl<'a> GraphIndex<'a> {
    /// Builds a new `GraphIndex` for the provided design.
    pub fn build(design: &'a Design) -> Self {
        let start = std::time::Instant::now();

        let cell_refs_topo = Self::build_cell_refs_topo(design);
        let cell_registry = CellRegistry::build(&cell_refs_topo);
        let connectivity =
            ConnectivityGraph::build(design, &cell_refs_topo, cell_registry.cell_id_map());
        let io_mapping = IoMapping::build(
            &cell_refs_topo,
            connectivity.fanin_map(),
            connectivity.fanout_map(),
        );

        tracing::debug!(
            "graph index built in {:?} for {} cells",
            start.elapsed(),
            cell_registry.len()
        );

        GraphIndex {
            cell_registry,
            connectivity,
            io_mapping,
        }
    }

    /// Returns the total number of cells in the index.
    #[must_use]
    pub const fn num_cells(&self) -> usize {
        self.cell_registry.len()
    }

    /// Internal helper to collect cell references in reverse topological order.
    fn build_cell_refs_topo(design: &'a Design) -> Vec<CellRef<'a>> {
        design
            .iter_cells_topo()
            .rev()
            .filter(|cell_ref| {
                let node_type = CellKind::from(cell_ref.get().as_ref());
                !matches!(node_type, CellKind::Name)
            })
            .collect()
    }

    /// Retrieves the internal index for a cell by its debug identifier.
    #[must_use]
    pub fn get_cell_index_by_debug_id(&self, debug_index: usize) -> Option<CellId> {
        self.cell_registry.cell_id_map().get(&debug_index).copied()
    }

    /// Returns the set of cell indices in the immediate fan-out of the specified cell.
    #[must_use]
    pub fn fanout_set(&self, cell_idx: CellId) -> &HashSet<CellId> {
        self.connectivity.fanout_indices_set(cell_idx)
    }

    /// Returns the set of cell indices in the immediate fan-in of the specified cell.
    #[must_use]
    pub fn fanin_set(&self, cell_idx: CellId) -> &HashSet<CellId> {
        self.connectivity.fanin_indices_set(cell_idx)
    }

    /// Returns the fan-out cell indices paired with their source pin indices.
    #[must_use]
    pub fn fanout_with_ports(&self, cell_idx: CellId) -> Option<&[(CellId, usize)]> {
        self.connectivity.fanout_indices(cell_idx)
    }

    /// Returns the fan-in cell indices paired with their source pin indices.
    #[must_use]
    pub fn fanin_with_ports(&self, cell_idx: CellId) -> Option<&[(CellId, usize)]> {
        self.connectivity.fanin_indices(cell_idx)
    }

    /// Returns the intersection of fan-outs for all cells in the fan-in of the specified cell.
    #[must_use]
    pub fn get_intersect_fanout_of_fanin(&self, cell_idx: CellId) -> &HashSet<CellId> {
        self.connectivity
            .get_intersect_fanout_of_fanin_indices(cell_idx)
    }

    /// Returns a map of input port names to their fan-out cell indices.
    #[must_use]
    pub fn get_input_fanout_by_name_indices(&self) -> &HashMap<String, Vec<(CellId, usize)>> {
        self.io_mapping.input_fanout_by_name_map()
    }

    /// Returns a map of output port names to their fan-in cell indices.
    #[must_use]
    pub fn get_output_fanin_by_name_indices(&self) -> &HashMap<String, Vec<(CellId, usize)>> {
        self.io_mapping.output_fanin_by_name_map()
    }

    /// Resolves a `CellId` to its `CellWrapper`.
    #[must_use]
    pub fn get_cell_by_index(&self, index: CellId) -> &CellWrapper<'a> {
        self.cell_registry.get_cell_by_index(index)
    }

    /// Returns indices for cells of a specific type.
    #[must_use]
    pub fn cells_of_type_indices(&self, node_type: CellKind) -> &[CellId] {
        self.cell_registry.cells_of_type_indices(node_type)
    }

    /// Retrieves a cell wrapper by its unique debug identifier.
    #[must_use]
    pub fn get_cell_by_id(&self, id: usize) -> Option<&CellWrapper<'a>> {
        self.get_cell_index_by_debug_id(id)
            .map(|idx| self.cell_registry.get_cell_by_index(idx))
    }

    /// Checks if the cell with `from_id` physically drives the cell with `to_id`.
    ///
    /// This is used by the query engine to validate structural constraints between
    /// matched components.
    #[must_use]
    pub fn is_connected(&self, from_id: u64, to_id: u64) -> bool {
        // 1. Map external ID (u64) to internal CellId
        let from_idx = match self.get_cell_index_by_debug_id(from_id as usize) {
            Some(idx) => idx,
            None => return false, // Source cell not found in graph
        };

        let to_idx = match self.get_cell_index_by_debug_id(to_id as usize) {
            Some(idx) => idx,
            None => return false, // Target cell not found in graph
        };

        // 2. Check adjacency list in ConnectivityGraph
        // We iterate the fanout of 'from' to see if 'to' is present.
        if let Some(fanout) = self.connectivity.fanout_indices(from_idx) {
            return fanout.iter().any(|(target, _)| *target == to_idx);
        }

        false
    }
}
