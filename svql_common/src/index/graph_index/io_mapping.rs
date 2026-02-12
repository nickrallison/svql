//! Primary I/O mapping for module boundaries.
//!
//! Manages the translation between named module ports and the
//! internal logic cells that drive or are driven by them.

use crate::cell::GraphNodeIdx;
use crate::*;
use prjunnamed_netlist::{Cell, CellRef};

/// Manages the connection between primary module ports and internal logic cells.
#[derive(Clone, Debug)]
pub struct IoMapping {
    /// Maps input names to their fan-out cells
    input_fanout_by_name: HashMap<String, Vec<(GraphNodeIdx, usize)>>,
    /// Maps output names to their fan-in cells
    output_fanin_by_name: HashMap<String, Vec<(GraphNodeIdx, usize)>>,
}

impl IoMapping {
    /// Builds the I/O mapping by inspecting module boundaries and internal connectivity.
    #[must_use]
    pub fn build(
        cell_refs_topo: &[CellRef<'_>],
        fanin_map: &HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>>,
        fanout_map: &HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>>,
    ) -> Self {
        let input_fanout_by_name = Self::build_input_fanout_by_name(cell_refs_topo, fanout_map);
        let output_fanin_by_name = Self::build_output_fanin_by_name(cell_refs_topo, fanin_map);

        Self {
            input_fanout_by_name,
            output_fanin_by_name,
        }
    }

    fn build_input_fanout_by_name(
        cell_refs_topo: &[CellRef<'_>],
        fanout_map: &HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>>,
    ) -> HashMap<String, Vec<(GraphNodeIdx, usize)>> {
        fanout_map
            .iter()
            .filter_map(|(cell_index, fanout_list)| {
                if let Cell::Input(name, _) = cell_refs_topo[cell_index.as_usize()].get().as_ref() {
                    Some((name.clone(), fanout_list.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn build_output_fanin_by_name(
        cell_refs_topo: &[CellRef<'_>],
        fanin_map: &HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>>,
    ) -> HashMap<String, Vec<(GraphNodeIdx, usize)>> {
        fanin_map
            .iter()
            .filter_map(|(cell_index, fanin_list)| {
                if let Cell::Output(name, _) = cell_refs_topo[cell_index.as_usize()].get().as_ref()
                {
                    Some((name.clone(), fanin_list.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Access the map of input names to the internal cells they drive.
    #[must_use]
    pub const fn input_fanout_by_name_map(&self) -> &HashMap<String, Vec<(GraphNodeIdx, usize)>> {
        &self.input_fanout_by_name
    }

    /// Access the map of output names to the internal cells that drive them.
    #[must_use]
    pub const fn output_fanin_by_name_map(&self) -> &HashMap<String, Vec<(GraphNodeIdx, usize)>> {
        &self.output_fanin_by_name
    }
}
