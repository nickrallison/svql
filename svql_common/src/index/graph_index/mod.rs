//! Graph indexing and connectivity analysis.
//!
//! Provides efficient data structures for querying graph connectivity,
//! cell types, and I/O mappings.

mod cell_registry;
mod connectivity_graph;
mod io_mapping;

use std::sync::Arc;

pub use cell_registry::CellRegistry;
pub use connectivity_graph::ConnectivityGraph;
pub use io_mapping::IoMapping;
use tracing::info;

use crate::cell::{CellKind, CellWrapper};
use crate::*;
use prjunnamed_netlist::{CellRef, Design};

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
        info!("Building graph index for design");
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

        tracing::info!(
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
    pub fn get_cell_index_by_debug_id(&self, debug_index: usize) -> Option<GraphNodeIdx> {
        self.cell_registry.cell_id_map().get(&debug_index).copied()
    }

    /// Returns the set of cell indices in the immediate fan-out of the specified cell.
    #[must_use]
    pub fn fanout_set(&self, cell_idx: GraphNodeIdx) -> &HashSet<GraphNodeIdx> {
        self.connectivity.fanout_indices_set(cell_idx)
    }

    /// Returns the set of cell indices in the immediate fan-in of the specified cell.
    #[must_use]
    pub fn fanin_set(&self, cell_idx: GraphNodeIdx) -> &HashSet<GraphNodeIdx> {
        self.connectivity.fanin_indices_set(cell_idx)
    }

    /// Returns the fan-out cell indices paired with their source pin indices.
    #[must_use]
    pub fn fanout_with_ports(&self, cell_idx: GraphNodeIdx) -> Option<&[(GraphNodeIdx, usize)]> {
        self.connectivity
            .fanout_map()
            .get(&cell_idx)
            .map(std::vec::Vec::as_slice)
    }

    /// Returns the fan-in cell indices paired with their source pin indices.
    #[must_use]
    pub fn fanin_with_ports(&self, cell_idx: GraphNodeIdx) -> Option<&[(GraphNodeIdx, usize)]> {
        self.connectivity
            .fanin_map()
            .get(&cell_idx)
            .map(std::vec::Vec::as_slice)
    }

    /// Returns the intersection of fan-outs for all cells in the fan-in of the specified cell.
    #[must_use]
    pub fn get_intersect_fanout_of_fanin(
        &self,
        cell_idx: GraphNodeIdx,
    ) -> Arc<HashSet<GraphNodeIdx>> {
        self.connectivity
            .get_intersect_fanout_of_fanin_indices(cell_idx)
    }

    /// Returns a map of input port names to their fan-out cell indices.
    #[must_use]
    pub fn get_input_fanout_by_name_indices(&self) -> &HashMap<String, Vec<(GraphNodeIdx, usize)>> {
        self.io_mapping.input_fanout_by_name_map()
    }

    /// Returns a map of output port names to their fan-in cell indices.
    #[must_use]
    pub fn get_output_fanin_by_name_indices(&self) -> &HashMap<String, Vec<(GraphNodeIdx, usize)>> {
        self.io_mapping.output_fanin_by_name_map()
    }

    // --- Job A: Fast Topology (For Solver) ---

    pub fn fanout(&self, node: GraphNodeIdx) -> &[GraphNodeIdx] {
        self.connectivity.fanout_indices(node).unwrap_or(&[])
    }

    pub fn fanin(&self, node: GraphNodeIdx) -> &[GraphNodeIdx] {
        self.connectivity.fanin_indices(node).unwrap_or(&[])
    }

    // --- Job B: Translation (For Storage Handoff) ---

    /// Maps a local solver node to its stable physical ID.
    pub fn resolve_physical(&self, node: GraphNodeIdx) -> PhysicalCellId {
        self.get_cell_by_index(node).debug_index()
    }

    /// Finds a node in the current graph by its physical ID.
    pub fn resolve_node(&self, physical: PhysicalCellId) -> Option<GraphNodeIdx> {
        // Here we access the internal int of physical only inside the safe boundary
        let key = physical.storage_key() as usize;
        self.cell_registry.cell_id_map().get(&key).copied()
    }

    // --- Job C: Capabilities (For Pruning) ---

    pub fn node_kind(&self, node: GraphNodeIdx) -> CellKind {
        self.get_cell_by_index(node).cell_type()
    }

    pub fn candidates(&self, kind: CellKind) -> &[GraphNodeIdx] {
        self.cell_registry.cells_of_type_indices(kind)
    }

    /// Helper to get a port name directly from a physical ID.
    /// Useful for reporting logic in `svql_query`.
    pub fn get_port_name(&self, physical: PhysicalCellId) -> Option<&str> {
        let node = self.resolve_node(physical)?;
        let wrapper = self.get_cell_by_index(node);
        wrapper.input_name().or_else(|| wrapper.output_name())
    }

    /// Resolves a `CellId` to its `CellWrapper`.
    #[must_use]
    pub fn get_cell_by_index(&self, index: GraphNodeIdx) -> &CellWrapper<'a> {
        self.cell_registry.get_cell_by_index(index)
    }

    /// Returns indices for cells of a specific type.
    #[must_use]
    pub fn cells_of_type_indices(&self, node_type: CellKind) -> &[GraphNodeIdx] {
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
        // 1. Map external ID (u64) to internal GraphNodeIdx
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
        if let Some(fanout) = self.connectivity.fanout_map().get(&from_idx) {
            return fanout.iter().any(|(target, _)| *target == to_idx);
        }

        false
    }
}
