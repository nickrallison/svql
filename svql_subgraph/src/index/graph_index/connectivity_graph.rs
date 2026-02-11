use std::sync::Arc;

use crate::cell::GraphNodeIdx;
use dashmap::DashMap;
use prjunnamed_netlist::{CellRef, Design};
use svql_common::*;

type FaninMap = HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>>;
type FanoutMap = HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>>;

#[derive(Clone, Debug)]
pub struct ConnectivityGraph {
    /// Maps each cell to its fan-in cells with port information
    fanin_map: FaninMap,
    /// Maps each cell to its fan-out cells with port information
    fanout_map: FanoutMap,
    /// Precomputed fan-out sets (no port info) for fast queries
    fanout_sets: HashMap<GraphNodeIdx, HashSet<GraphNodeIdx>>,
    /// Precomputed fan-in sets (no port info) for fast queries
    fanin_sets: HashMap<GraphNodeIdx, HashSet<GraphNodeIdx>>,
    /// Precomputed fan-out indices (no port info) for fast queries
    fanout_indices: HashMap<GraphNodeIdx, Vec<GraphNodeIdx>>,
    /// Precomputed fan-in indices (no port info) for fast queries
    fanin_indices: HashMap<GraphNodeIdx, Vec<GraphNodeIdx>>,
    /// Precomputed intersection of fanout of fanin
    intersect_fanout_of_fanin: Arc<DashMap<GraphNodeIdx, Arc<HashSet<GraphNodeIdx>>>>,
}

impl ConnectivityGraph {
    #[must_use]
    pub fn build(
        design: &Design,
        cell_refs_topo: &[CellRef<'_>],
        cell_id_map: &HashMap<usize, GraphNodeIdx>,
    ) -> Self {
        let (fanin_map, fanout_map) =
            Self::build_fanin_fanout_maps(design, cell_refs_topo, cell_id_map);

        // Precompute fanout and fanin sets
        let fanout_sets = Self::precompute_sets(&fanout_map);
        let fanin_sets = Self::precompute_sets(&fanin_map);
        let fanout_indices = Self::precompute_indices(&fanout_map);
        let fanin_indices = Self::precompute_indices(&fanin_map);

        Self {
            fanin_map,
            fanout_map,
            fanout_sets,
            fanin_sets,
            fanout_indices,
            fanin_indices,
            intersect_fanout_of_fanin: Arc::new(DashMap::new()),
        }
    }

    fn compute_single_intersect(&self, cell_idx: GraphNodeIdx) -> HashSet<GraphNodeIdx> {
        let Some(fanin_list) = self.fanin_map.get(&cell_idx) else {
            return HashSet::default();
        };

        if fanin_list.is_empty() {
            return HashSet::default();
        }

        let fanout_sets_of_fanins: Vec<&HashSet<GraphNodeIdx>> = fanin_list
            .iter()
            .filter_map(|(idx, _)| self.fanout_sets.get(idx))
            .collect();

        if fanout_sets_of_fanins.is_empty() {
            return HashSet::default();
        }

        let mut result = fanout_sets_of_fanins[0].clone();
        for set in &fanout_sets_of_fanins[1..] {
            result = &result & *set;
        }

        result
    }

    fn build_fanin_fanout_maps(
        design: &Design,
        cell_refs_topo: &[CellRef<'_>],
        cell_id_map: &HashMap<usize, GraphNodeIdx>,
    ) -> (FaninMap, FanoutMap) {
        let mut fanout_map: FanoutMap = HashMap::default();
        let mut fanin_map: FaninMap = HashMap::default();

        for sink_ref in cell_refs_topo.iter() {
            let sink_idx = *cell_id_map
                .get(&sink_ref.debug_index())
                .expect("Sink cell should be in map");
            sink_ref.visit(|net| {
                if let Ok((source_ref, source_pin_idx)) = design.find_cell(net) {
                    let source_ref_idx = *cell_id_map
                        .get(&source_ref.debug_index())
                        .expect("Source cell should be in map");

                    fanout_map
                        .entry(source_ref_idx)
                        .or_default()
                        .push((sink_idx, source_pin_idx));
                    fanin_map
                        .entry(sink_idx)
                        .or_default()
                        .push((source_ref_idx, source_pin_idx));
                }
            });
        }
        (fanin_map, fanout_map)
    }

    fn precompute_sets(
        map: &HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>>,
    ) -> HashMap<GraphNodeIdx, HashSet<GraphNodeIdx>> {
        map.iter()
            .map(|(idx, vec)| {
                let set = vec.iter().map(|(cell_idx, _)| *cell_idx).collect();
                (*idx, set)
            })
            .collect()
    }

    fn precompute_indices(
        map: &HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>>,
    ) -> HashMap<GraphNodeIdx, Vec<GraphNodeIdx>> {
        map.iter()
            .map(|(idx, vec)| {
                let indices = vec.iter().map(|(cell_idx, _)| *cell_idx).collect();
                (*idx, indices)
            })
            .collect()
    }

    #[must_use]
    pub fn fanout_indices(&self, cell_idx: GraphNodeIdx) -> Option<&[GraphNodeIdx]> {
        self.fanout_indices
            .get(&cell_idx)
            .map(std::vec::Vec::as_slice)
    }

    #[must_use]
    pub fn fanin_indices(&self, cell_idx: GraphNodeIdx) -> Option<&[GraphNodeIdx]> {
        self.fanin_indices
            .get(&cell_idx)
            .map(std::vec::Vec::as_slice)
    }

    #[must_use]
    pub fn fanout_indices_set(&self, cell_idx: GraphNodeIdx) -> &HashSet<GraphNodeIdx> {
        self.fanout_sets.get(&cell_idx).unwrap_or_else(|| {
            static EMPTY: std::sync::OnceLock<HashSet<GraphNodeIdx>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(|| HashSet::default())
        })
    }

    #[must_use]
    pub fn fanin_indices_set(&self, cell_idx: GraphNodeIdx) -> &HashSet<GraphNodeIdx> {
        self.fanin_sets.get(&cell_idx).unwrap_or_else(|| {
            static EMPTY: std::sync::OnceLock<HashSet<GraphNodeIdx>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(|| HashSet::default())
        })
    }

    /// Get intersection (computes on-demand, caches result)
    #[must_use]
    pub fn get_intersect_fanout_of_fanin_indices(
        &self,
        cell_idx: GraphNodeIdx,
    ) -> Arc<HashSet<GraphNodeIdx>> {
        self.intersect_fanout_of_fanin
            .entry(cell_idx)
            .or_insert_with(|| Arc::new(self.compute_single_intersect(cell_idx)))
            .clone()
    }

    #[must_use]
    pub const fn fanin_map(&self) -> &HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>> {
        &self.fanin_map
    }

    #[must_use]
    pub const fn fanout_map(&self) -> &HashMap<GraphNodeIdx, Vec<(GraphNodeIdx, usize)>> {
        &self.fanout_map
    }
}
