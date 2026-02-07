use crate::cell::CellIndex;
use prjunnamed_netlist::{CellRef, Design};
use svql_common::prelude::*;

type FaninMap = HashMap<CellIndex, Vec<(CellIndex, usize)>>;
type FanoutMap = HashMap<CellIndex, Vec<(CellIndex, usize)>>;

#[derive(Clone, Debug)]
pub struct ConnectivityGraph {
    /// Maps each cell to its fan-in cells with port information
    fanin_map: FaninMap,
    /// Maps each cell to its fan-out cells with port information
    fanout_map: FanoutMap,
    /// Precomputed fan-out sets (no port info) for fast queries
    fanout_sets: HashMap<CellIndex, HashSet<CellIndex>>,
    /// Precomputed fan-in sets (no port info) for fast queries
    fanin_sets: HashMap<CellIndex, HashSet<CellIndex>>,
    /// Precomputed intersection of fanout of fanin
    intersect_fanout_of_fanin: HashMap<CellIndex, HashSet<CellIndex>>,
}

impl ConnectivityGraph {
    #[must_use]
    pub fn build(
        design: &Design,
        cell_refs_topo: &[CellRef<'_>],
        cell_id_map: &HashMap<usize, CellIndex>,
    ) -> Self {
        let (fanin_map, fanout_map) =
            Self::build_fanin_fanout_maps(design, cell_refs_topo, cell_id_map);

        // Precompute fanout and fanin sets
        let fanout_sets = Self::precompute_sets(&fanout_map);
        let fanin_sets = Self::precompute_sets(&fanin_map);

        // Precompute intersection of fanout of fanin
        let intersect_fanout_of_fanin =
            Self::precompute_intersect_fanout_of_fanin(&fanin_map, &fanout_sets);

        Self {
            fanin_map,
            fanout_map,
            fanout_sets,
            fanin_sets,
            intersect_fanout_of_fanin,
        }
    }

    fn build_fanin_fanout_maps(
        design: &Design,
        cell_refs_topo: &[CellRef<'_>],
        cell_id_map: &HashMap<usize, CellIndex>,
    ) -> (FaninMap, FanoutMap) {
        let mut fanout_map: FanoutMap = HashMap::new();
        let mut fanin_map: FaninMap = HashMap::new();

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
        map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<CellIndex, HashSet<CellIndex>> {
        map.iter()
            .map(|(idx, vec)| {
                let set = vec.iter().map(|(cell_idx, _)| *cell_idx).collect();
                (*idx, set)
            })
            .collect()
    }

    fn precompute_intersect_fanout_of_fanin(
        fanin_map: &FaninMap,
        fanout_sets: &HashMap<CellIndex, HashSet<CellIndex>>,
    ) -> HashMap<CellIndex, HashSet<CellIndex>> {
        fanin_map
            .iter()
            .filter_map(|(cell_idx, fanin_list)| {
                if fanin_list.is_empty() {
                    return None;
                }

                let fanout_sets_of_fanins: Vec<&HashSet<CellIndex>> = fanin_list
                    .iter()
                    .filter_map(|(idx, _)| fanout_sets.get(idx))
                    .collect();

                if fanout_sets_of_fanins.is_empty() {
                    return None;
                }

                let mut result = fanout_sets_of_fanins[0].clone();
                for set in &fanout_sets_of_fanins[1..] {
                    result = &result & *set;
                }

                Some((*cell_idx, result))
            })
            .collect()
    }

    #[must_use]
    pub fn fanout_indices(&self, cell_idx: CellIndex) -> Option<&[(CellIndex, usize)]> {
        self.fanout_map.get(&cell_idx).map(std::vec::Vec::as_slice)
    }

    #[must_use]
    pub fn fanin_indices(&self, cell_idx: CellIndex) -> Option<&[(CellIndex, usize)]> {
        self.fanin_map.get(&cell_idx).map(std::vec::Vec::as_slice)
    }

    #[must_use]
    pub fn fanout_indices_set(&self, cell_idx: CellIndex) -> &HashSet<CellIndex> {
        self.fanout_sets.get(&cell_idx).unwrap_or_else(|| {
            static EMPTY: std::sync::OnceLock<HashSet<CellIndex>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(HashSet::new)
        })
    }

    #[must_use]
    pub fn fanin_indices_set(&self, cell_idx: CellIndex) -> &HashSet<CellIndex> {
        self.fanin_sets.get(&cell_idx).unwrap_or_else(|| {
            static EMPTY: std::sync::OnceLock<HashSet<CellIndex>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(HashSet::new)
        })
    }

    #[must_use]
    pub fn get_intersect_fanout_of_fanin_indices(
        &self,
        cell_idx: CellIndex,
    ) -> &HashSet<CellIndex> {
        self.intersect_fanout_of_fanin
            .get(&cell_idx)
            .unwrap_or_else(|| {
                static EMPTY: std::sync::OnceLock<HashSet<CellIndex>> = std::sync::OnceLock::new();
                EMPTY.get_or_init(HashSet::new)
            })
    }

    #[must_use]
    pub const fn fanin_map(&self) -> &HashMap<CellIndex, Vec<(CellIndex, usize)>> {
        &self.fanin_map
    }

    #[must_use]
    pub const fn fanout_map(&self) -> &HashMap<CellIndex, Vec<(CellIndex, usize)>> {
        &self.fanout_map
    }
}
