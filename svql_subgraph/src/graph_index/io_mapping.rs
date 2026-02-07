use crate::cell::CellIndex;
use prjunnamed_netlist::{Cell, CellRef};
use svql_common::prelude::*;

#[derive(Clone, Debug)]
pub struct IoMapping {
    /// Maps input names to their fan-out cells
    input_fanout_by_name: HashMap<String, Vec<(CellIndex, usize)>>,
    /// Maps output names to their fan-in cells
    output_fanin_by_name: HashMap<String, Vec<(CellIndex, usize)>>,
}

impl IoMapping {
    #[must_use]
    pub fn build(
        cell_refs_topo: &[CellRef<'_>],
        fanin_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
        fanout_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
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
        fanout_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<String, Vec<(CellIndex, usize)>> {
        fanout_map
            .iter()
            .filter_map(|(cell_index, fanout_list)| {
                if let Cell::Input(name, _) = cell_refs_topo[cell_index.index()].get().as_ref() {
                    Some((name.clone(), fanout_list.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn build_output_fanin_by_name(
        cell_refs_topo: &[CellRef<'_>],
        fanin_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<String, Vec<(CellIndex, usize)>> {
        fanin_map
            .iter()
            .filter_map(|(cell_index, fanin_list)| {
                if let Cell::Output(name, _) = cell_refs_topo[cell_index.index()].get().as_ref() {
                    Some((name.clone(), fanin_list.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use]
    pub const fn input_fanout_by_name_map(&self) -> &HashMap<String, Vec<(CellIndex, usize)>> {
        &self.input_fanout_by_name
    }

    #[must_use]
    pub const fn output_fanin_by_name_map(&self) -> &HashMap<String, Vec<(CellIndex, usize)>> {
        &self.output_fanin_by_name
    }
}
