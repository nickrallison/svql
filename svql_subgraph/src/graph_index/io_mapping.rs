use super::CellRegistry;
use crate::cell::{CellIndex, CellWrapper};
use prjunnamed_netlist::Cell;
use std::collections::HashMap;

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
        cells_topo: &[CellWrapper<'_>],
        fanin_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
        fanout_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> Self {
        let input_fanout_by_name = Self::build_input_fanout_by_name(cells_topo, fanout_map);
        let output_fanin_by_name = Self::build_output_fanin_by_name(cells_topo, fanin_map);

        Self {
            input_fanout_by_name,
            output_fanin_by_name,
        }
    }

    fn build_input_fanout_by_name(
        cells_topo: &[CellWrapper<'_>],
        fanout_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<String, Vec<(CellIndex, usize)>> {
        fanout_map
            .iter()
            .filter_map(|(cell_index, fanout_list)| {
                if let Cell::Input(name, _) = cells_topo[cell_index.index()].get() {
                    Some((name.clone(), fanout_list.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn build_output_fanin_by_name(
        cells_topo: &[CellWrapper<'_>],
        fanin_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<String, Vec<(CellIndex, usize)>> {
        fanin_map
            .iter()
            .filter_map(|(cell_index, fanin_list)| {
                if let Cell::Output(name, _) = cells_topo[cell_index.index()].get() {
                    Some((name.clone(), fanin_list.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use] 
    pub fn get_input_fanout_by_name<'a>(
        &self,
        registry: &CellRegistry<'a>,
    ) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        self.input_fanout_by_name
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter()
                        .map(|(c, pin)| (registry.get_cell_by_index(*c).clone(), *pin))
                        .collect(),
                )
            })
            .collect()
    }

    #[must_use] 
    pub fn get_output_fanin_by_name<'a>(
        &self,
        registry: &CellRegistry<'a>,
    ) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        self.output_fanin_by_name
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter()
                        .map(|(c, pin)| (registry.get_cell_by_index(*c).clone(), *pin))
                        .collect(),
                )
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
