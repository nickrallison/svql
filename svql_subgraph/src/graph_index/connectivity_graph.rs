use super::CellRegistry;
use crate::cell::{CellIndex, CellWrapper};
use prjunnamed_netlist::{CellRef, Design};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct ConnectivityGraph {
    /// Maps each cell to its fan-in cells with port information
    fanin_map: HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    /// Maps each cell to its fan-out cells with port information
    fanout_map: HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    /// Maps each cell to its fan-out cells (no port information)
    clean_fanout_map: HashMap<CellIndex, HashSet<CellIndex>>,
    /// Maps each cell to its fan-in cells (no port information)
    clean_fanin_map: HashMap<CellIndex, HashSet<CellIndex>>,
}

impl ConnectivityGraph {
    pub fn build<'a>(
        design: &'a Design,
        cell_refs_topo: &[CellRef<'a>],
        cell_id_map: &HashMap<usize, CellIndex>,
    ) -> Self {
        let (fanin_map, fanout_map) =
            Self::build_fanin_fanout_maps(design, cell_refs_topo, cell_id_map);
        let clean_fanout_map = Self::build_clean_map(&fanout_map);
        let clean_fanin_map = Self::build_clean_map(&fanin_map);

        ConnectivityGraph {
            fanin_map,
            fanout_map,
            clean_fanout_map,
            clean_fanin_map,
        }
    }

    fn build_fanin_fanout_maps<'a>(
        design: &'a Design,
        cell_refs_topo: &[CellRef<'a>],
        cell_id_map: &HashMap<usize, CellIndex>,
    ) -> (
        HashMap<CellIndex, Vec<(CellIndex, usize)>>,
        HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) {
        let mut fanout_map: HashMap<CellIndex, Vec<(CellIndex, usize)>> = HashMap::new();
        let mut fanin_map: HashMap<CellIndex, Vec<(CellIndex, usize)>> = HashMap::new();

        for sink_ref in cell_refs_topo.iter().cloned() {
            let sink_wrapper: CellWrapper<'a> = sink_ref.into();
            sink_ref.visit(|net| {
                if let Ok((source_ref, source_pin_idx)) = design.find_cell(net) {
                    let source_ref_idx = cell_id_map
                        .get(&source_ref.debug_index())
                        .expect("Source cell should be in map")
                        .clone();

                    let sink_wrapper_idx = cell_id_map
                        .get(&sink_wrapper.debug_index())
                        .expect("Sink cell should be in map")
                        .clone();

                    fanout_map
                        .entry(source_ref_idx)
                        .or_default()
                        .push((sink_wrapper_idx, source_pin_idx));
                    fanin_map
                        .entry(sink_wrapper_idx)
                        .or_default()
                        .push((source_ref_idx, source_pin_idx));
                }
            });
        }
        (fanin_map, fanout_map)
    }

    fn build_clean_map(
        fanout_fanin_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<CellIndex, HashSet<CellIndex>> {
        fanout_fanin_map
            .iter()
            .map(|(key, vec)| {
                let set: HashSet<CellIndex> = vec.iter().map(|(c, _)| *c).collect();
                (*key, set)
            })
            .collect()
    }

    pub fn fanout_set<'a>(
        &self,
        cell: &CellWrapper<'a>,
        registry: &CellRegistry<'a>,
    ) -> Option<HashSet<CellWrapper<'a>>> {
        let idx = registry.get_cell_index(cell)?;
        self.clean_fanout_map.get(&idx).map(|v| {
            v.iter()
                .map(|idx| registry.get_cell_by_index(*idx).clone())
                .collect()
        })
    }

    pub fn fanin_set<'a>(
        &self,
        cell: &CellWrapper<'a>,
        registry: &CellRegistry<'a>,
    ) -> Option<HashSet<CellWrapper<'a>>> {
        let idx = registry.get_cell_index(cell)?;
        self.clean_fanin_map.get(&idx).map(|v| {
            v.iter()
                .map(|idx| registry.get_cell_by_index(*idx).clone())
                .collect()
        })
    }

    pub fn fanout_with_ports<'a>(
        &self,
        cell: &CellWrapper<'a>,
        registry: &CellRegistry<'a>,
    ) -> Option<Vec<(CellWrapper<'a>, usize)>> {
        let idx = registry.get_cell_index(cell)?;
        self.fanout_map.get(&idx).map(|v| {
            v.iter()
                .map(|(idx, pin)| (registry.get_cell_by_index(*idx).clone(), *pin))
                .collect()
        })
    }

    pub fn fanin_with_ports<'a>(
        &self,
        cell: &CellWrapper<'a>,
        registry: &CellRegistry<'a>,
    ) -> Option<Vec<(CellWrapper<'a>, usize)>> {
        let idx = registry.get_cell_index(cell)?;
        self.fanin_map.get(&idx).map(|v| {
            v.iter()
                .map(|(idx, pin)| (registry.get_cell_by_index(*idx).clone(), *pin))
                .collect()
        })
    }

    pub fn get_intersect_fanout_of_fanin<'a>(
        &self,
        cell: &CellWrapper<'a>,
        registry: &CellRegistry<'a>,
    ) -> HashSet<CellWrapper<'a>> {
        let mut fanin: Vec<CellWrapper<'a>> = self
            .fanin_set(cell, registry)
            .map(|s| s.into_iter().collect())
            .unwrap_or_default();

        if fanin.is_empty() {
            return HashSet::new();
        }

        let first_fanin = fanin.remove(0);
        let initial_fanout = self.fanout_set(&first_fanin, registry).unwrap_or_default();

        fanin
            .iter()
            .map(|c| self.fanout_set(c, registry).unwrap_or_default())
            .fold(initial_fanout, |acc: HashSet<CellWrapper<'a>>, hs| {
                acc.intersection(&hs).cloned().collect()
            })
    }

    pub fn fanin_map(&self) -> &HashMap<CellIndex, Vec<(CellIndex, usize)>> {
        &self.fanin_map
    }

    pub fn fanout_map(&self) -> &HashMap<CellIndex, Vec<(CellIndex, usize)>> {
        &self.fanout_map
    }
}
