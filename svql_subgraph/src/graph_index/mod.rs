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

use crate::cell::{CellIndex, CellKind, CellWrapper};
use prjunnamed_netlist::{CellRef, Design};
use std::collections::{HashMap, HashSet};

/// An index over a design graph providing fast access to connectivity and cell data.
#[derive(Clone, Debug)]
pub struct GraphIndex<'a> {
    cell_registry: CellRegistry<'a>,
    connectivity: ConnectivityGraph,
    io_mapping: IoMapping,
}

impl<'a> GraphIndex<'a> {
    /// Builds a new GraphIndex for the provided design.
    pub fn build(design: &'a Design) -> Self {
        let start = std::time::Instant::now();

        let cell_refs_topo = Self::build_cell_refs_topo(design);
        let cell_registry = CellRegistry::build(&cell_refs_topo);
        let connectivity =
            ConnectivityGraph::build(design, &cell_refs_topo, cell_registry.cell_id_map());
        let io_mapping = IoMapping::build(
            cell_registry.cells_topo(),
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
    pub fn num_cells(&self) -> usize {
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

    /// Returns an iterator over cells of a specific type.
    pub fn cells_of_type_iter(
        &self,
        node_type: CellKind,
    ) -> Option<impl Iterator<Item = &CellWrapper<'a>>> {
        self.cell_registry.cells_of_type_iter(node_type)
    }

    /// Returns a slice of all cells in topological order.
    pub fn cells_topo(&self) -> &[CellWrapper<'a>] {
        self.cell_registry.cells_topo()
    }

    /// Retrieves the internal index for a given cell wrapper.
    fn get_cell_index(&self, cell: &CellWrapper<'a>) -> Option<CellIndex> {
        self.cell_registry.get_cell_index(cell)
    }

    /// Returns the set of cells in the immediate fan-out of the specified cell.
    pub fn fanout_set(&self, cell: &CellWrapper<'a>) -> Option<HashSet<CellWrapper<'a>>> {
        let idx = self.get_cell_index(cell)?;
        let indices_set = self.connectivity.fanout_indices_set(idx);
        let cells: HashSet<CellWrapper<'a>> = indices_set
            .into_iter()
            .map(|idx| self.cell_registry.get_cell_by_index(idx).clone())
            .collect();
        Some(cells)
    }

    /// Returns the set of cells in the immediate fan-in of the specified cell.
    pub fn fanin_set(&self, cell: &CellWrapper<'a>) -> Option<HashSet<CellWrapper<'a>>> {
        let idx = self.get_cell_index(cell)?;
        let indices_set = self.connectivity.fanin_indices_set(idx);
        let cells: HashSet<CellWrapper<'a>> = indices_set
            .into_iter()
            .map(|idx| self.cell_registry.get_cell_by_index(idx).clone())
            .collect();
        Some(cells)
    }

    /// Returns the fan-out cells paired with their source pin indices.
    pub fn fanout_with_ports(
        &self,
        cell: &CellWrapper<'a>,
    ) -> Option<Vec<(CellWrapper<'a>, usize)>> {
        let idx = self.get_cell_index(cell)?;
        let indices_with_ports = self.connectivity.fanout_indices(idx)?;
        Some(
            self.cell_registry
                .indices_with_ports_to_cells(indices_with_ports),
        )
    }

    /// Returns the fan-in cells paired with their source pin indices.
    pub fn fanin_with_ports(
        &self,
        cell: &CellWrapper<'a>,
    ) -> Option<Vec<(CellWrapper<'a>, usize)>> {
        let idx = self.get_cell_index(cell)?;
        let indices_with_ports = self.connectivity.fanin_indices(idx)?;
        Some(
            self.cell_registry
                .indices_with_ports_to_cells(indices_with_ports),
        )
    }

    /// Returns the intersection of fan-outs for all cells in the fan-in of the specified cell.
    pub fn get_intersect_fanout_of_fanin(
        &self,
        cell: &CellWrapper<'a>,
    ) -> HashSet<CellWrapper<'a>> {
        let Some(idx) = self.get_cell_index(cell) else {
            return HashSet::new();
        };

        let intersection_indices = self.connectivity.get_intersect_fanout_of_fanin_indices(idx);

        intersection_indices
            .into_iter()
            .map(|idx| self.cell_registry.get_cell_by_index(idx).clone())
            .collect()
    }

    /// Returns a map of input port names to their fan-out cells.
    pub fn get_input_fanout_by_name(&self) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        self.io_mapping
            .get_input_fanout_by_name(&self.cell_registry)
    }

    /// Returns a map of output port names to their fan-in cells.
    pub fn get_output_fanin_by_name(&self) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        self.io_mapping
            .get_output_fanin_by_name(&self.cell_registry)
    }

    /// Retrieves a cell wrapper by its unique debug identifier.
    pub fn get_cell_by_id(&self, id: usize) -> Option<&CellWrapper<'a>> {
        self.cell_registry
            .cell_id_map()
            .get(&id)
            .map(|idx| self.cell_registry.get_cell_by_index(*idx))
    }
}
