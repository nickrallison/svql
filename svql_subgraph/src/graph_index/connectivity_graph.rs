use crate::cell::{CellIndex, CellWrapper};
use prjunnamed_netlist::{CellRef, Design};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct ConnectivityGraph {
    /// Maps each cell to its fan-in cells with port information
    fanin_map: HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    /// Maps each cell to its fan-out cells with port information
    fanout_map: HashMap<CellIndex, Vec<(CellIndex, usize)>>,
}

impl ConnectivityGraph {
    pub fn build<'a>(
        design: &'a Design,
        cell_refs_topo: &[CellRef<'a>],
        cell_id_map: &HashMap<usize, CellIndex>,
    ) -> Self {
        let (fanin_map, fanout_map) =
            Self::build_fanin_fanout_maps(design, cell_refs_topo, cell_id_map);

        ConnectivityGraph {
            fanin_map,
            fanout_map,
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

    // Work with indices for better performance
    pub fn fanout_indices(&self, cell_idx: CellIndex) -> Option<&[(CellIndex, usize)]> {
        self.fanout_map.get(&cell_idx).map(|v| v.as_slice())
    }

    pub fn fanin_indices(&self, cell_idx: CellIndex) -> Option<&[(CellIndex, usize)]> {
        self.fanin_map.get(&cell_idx).map(|v| v.as_slice())
    }

    // Helper to get clean indices set
    pub fn fanout_indices_set(&self, cell_idx: CellIndex) -> HashSet<CellIndex> {
        self.fanout_map
            .get(&cell_idx)
            .map(|v| v.iter().map(|(idx, _)| *idx).collect())
            .unwrap_or_default()
    }

    pub fn fanin_indices_set(&self, cell_idx: CellIndex) -> HashSet<CellIndex> {
        self.fanin_map
            .get(&cell_idx)
            .map(|v| v.iter().map(|(idx, _)| *idx).collect())
            .unwrap_or_default()
    }

    // Optimized intersection using indices
    pub fn get_intersect_fanout_of_fanin_indices(&self, cell_idx: CellIndex) -> HashSet<CellIndex> {
        let Some(fanin_indices) = self.fanin_map.get(&cell_idx) else {
            return HashSet::new();
        };

        if fanin_indices.is_empty() {
            return HashSet::new();
        }

        let fanout_sets: Vec<HashSet<CellIndex>> = fanin_indices
            .iter()
            .map(|(idx, _)| self.fanout_indices_set(*idx))
            .collect();

        if fanout_sets.is_empty() {
            return HashSet::new();
        }

        let mut result = fanout_sets[0].clone();
        for set in &fanout_sets[1..] {
            result = &result & set;
        }
        result
    }

    pub fn fanin_map(&self) -> &HashMap<CellIndex, Vec<(CellIndex, usize)>> {
        &self.fanin_map
    }

    pub fn fanout_map(&self) -> &HashMap<CellIndex, Vec<(CellIndex, usize)>> {
        &self.fanout_map
    }
}
