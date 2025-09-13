mod cell_registry;
mod connectivity_graph;
mod io_mapping;

pub use cell_registry::CellRegistry;
pub use connectivity_graph::ConnectivityGraph;
pub use io_mapping::IoMapping;

use crate::cell::{CellIndex, CellKind, CellWrapper};
use prjunnamed_netlist::{CellRef, Design};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct GraphIndex<'a> {
    cell_registry: CellRegistry<'a>,
    connectivity: ConnectivityGraph,
    io_mapping: IoMapping,
}

impl<'a> GraphIndex<'a> {
    pub fn build(design: &'a Design) -> Self {
        // 1. Build cell registry first
        let cell_refs_topo = Self::build_cell_refs_topo(design);
        let cell_registry = CellRegistry::build(&cell_refs_topo);

        // 2. Build connectivity graph using the registry
        let connectivity =
            ConnectivityGraph::build(design, &cell_refs_topo, cell_registry.cell_id_map());

        // 3. Build I/O mapping using both registry and connectivity
        let io_mapping = IoMapping::build(
            cell_registry.cells_topo(),
            connectivity.fanin_map(),
            connectivity.fanout_map(),
        );

        GraphIndex {
            cell_registry,
            connectivity,
            io_mapping,
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
    pub fn cells_of_type(&self, node_type: CellKind) -> Vec<CellWrapper<'a>> {
        self.cell_registry.cells_of_type(node_type)
    }

    pub fn cells_topo(&self) -> &[CellWrapper<'a>] {
        self.cell_registry.cells_topo()
    }

    // Delegate to connectivity graph
    pub fn fanout_set(&self, cell: &CellWrapper<'a>) -> Option<HashSet<CellWrapper<'a>>> {
        self.connectivity.fanout_set(cell, &self.cell_registry)
    }

    pub fn fanin_set(&self, cell: &CellWrapper<'a>) -> Option<HashSet<CellWrapper<'a>>> {
        self.connectivity.fanin_set(cell, &self.cell_registry)
    }

    pub fn fanout_with_ports(
        &self,
        cell: &CellWrapper<'a>,
    ) -> Option<Vec<(CellWrapper<'a>, usize)>> {
        self.connectivity
            .fanout_with_ports(cell, &self.cell_registry)
    }

    pub fn fanin_with_ports(
        &self,
        cell: &CellWrapper<'a>,
    ) -> Option<Vec<(CellWrapper<'a>, usize)>> {
        self.connectivity
            .fanin_with_ports(cell, &self.cell_registry)
    }

    pub fn get_intersect_fanout_of_fanin(
        &self,
        cell: &CellWrapper<'a>,
    ) -> HashSet<CellWrapper<'a>> {
        self.connectivity
            .get_intersect_fanout_of_fanin(cell, &self.cell_registry)
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
