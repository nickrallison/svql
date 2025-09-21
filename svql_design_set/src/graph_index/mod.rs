mod cell_registry;
mod connectivity_graph;
mod io_mapping;
mod submodule;

pub use cell_registry::CellRegistry;
pub use connectivity_graph::ConnectivityGraph;
pub use io_mapping::IoMapping;
pub(crate) use submodule::{SubmoduleContainer, SubmoduleKey};

use crate::cell::{CellIndex, CellKind, CellWrapper};
use prjunnamed_netlist::{CellRef, Design};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct GraphIndex<'a> {
    cell_registry: CellRegistry<'a>,
    connectivity: ConnectivityGraph,
    io_mapping: IoMapping,
    submodules: SubmoduleContainer,
}

impl<'a> GraphIndex<'a> {
    pub fn build(design: &'a Design) -> Self {
        let cell_refs_topo = Self::build_cell_refs_topo(design);
        let cell_registry = CellRegistry::build(&cell_refs_topo);
        let connectivity =
            ConnectivityGraph::build(design, &cell_refs_topo, cell_registry.cell_id_map());
        let io_mapping = IoMapping::build(
            cell_registry.cells_topo(),
            connectivity.fanin_map(),
            connectivity.fanout_map(),
        );
        let submodules = SubmoduleContainer::build(design);

        GraphIndex {
            cell_registry,
            connectivity,
            io_mapping,
            submodules,
        }
    }

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

    // Delegate to cell registry
    pub fn cells_of_type_iter(
        &self,
        node_type: CellKind,
    ) -> Option<impl Iterator<Item = &CellWrapper<'a>>> {
        self.cell_registry.cells_of_type_iter(node_type)
    }

    pub fn cells_topo(&self) -> &[CellWrapper<'a>] {
        self.cell_registry.cells_topo()
    }

    // Helper to get cell index
    fn get_cell_index(&self, cell: &CellWrapper<'a>) -> Option<CellIndex> {
        self.cell_registry.get_cell_index(cell)
    }

    // Optimized methods that work with indices internally
    pub fn fanout_set(&self, cell: &CellWrapper<'a>) -> Option<HashSet<CellWrapper<'a>>> {
        let idx = self.get_cell_index(cell)?;
        let indices_set = self.connectivity.fanout_indices_set(idx);
        let cells: HashSet<CellWrapper<'a>> = indices_set
            .into_iter()
            .map(|idx| self.cell_registry.get_cell_by_index(idx).clone())
            .collect();
        Some(cells)
    }

    pub fn fanin_set(&self, cell: &CellWrapper<'a>) -> Option<HashSet<CellWrapper<'a>>> {
        let idx = self.get_cell_index(cell)?;
        let indices_set = self.connectivity.fanin_indices_set(idx);
        let cells: HashSet<CellWrapper<'a>> = indices_set
            .into_iter()
            .map(|idx| self.cell_registry.get_cell_by_index(idx).clone())
            .collect();
        Some(cells)
    }

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

    // Delegate to I/O mapping
    pub fn get_input_fanout_by_name(&self) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        self.io_mapping
            .get_input_fanout_by_name(&self.cell_registry)
    }

    pub fn get_output_fanin_by_name(&self) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        self.io_mapping
            .get_output_fanin_by_name(&self.cell_registry)
    }
}
