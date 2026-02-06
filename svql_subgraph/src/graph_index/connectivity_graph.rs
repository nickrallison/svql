use crate::cell::{CellIndex, CellWrapper};
use ahash::{AHashMap, AHashSet};
use prjunnamed_netlist::{CellRef, Design};

type FaninMap = AHashMap<CellIndex, Vec<(CellIndex, usize)>>;
type FanoutMap = AHashMap<CellIndex, Vec<(CellIndex, usize)>>;

#[derive(Clone, Debug)]
pub struct ConnectivityGraph<'a> {
    /// Maps each cell to its fan-in cells with port information
    fanin_map: FaninMap,
    /// Maps each cell to its fan-out cells with port information
    fanout_map: FanoutMap,
    /// Precomputed fan-out sets (no port info) for fast queries
    fanout_sets: AHashMap<CellIndex, AHashSet<CellIndex>>,
    /// Precomputed fan-in sets (no port info) for fast queries
    fanin_sets: AHashMap<CellIndex, AHashSet<CellIndex>>,
    /// Precomputed intersection of fanout of fanin
    intersect_fanout_of_fanin: AHashMap<CellIndex, AHashSet<CellIndex>>,
    /// Precomputed fan-out cell sets (CellWrapper) for zero-cost queries
    fanout_cell_sets: AHashMap<CellIndex, AHashSet<CellWrapper<'a>>>,
    /// Precomputed fan-in cell sets (CellWrapper) for zero-cost queries
    fanin_cell_sets: AHashMap<CellIndex, AHashSet<CellWrapper<'a>>>,
    /// Precomputed intersection of fanout of fanin (CellWrapper)
    intersect_fanout_of_fanin_cells: AHashMap<CellIndex, AHashSet<CellWrapper<'a>>>,
}

impl<'a> ConnectivityGraph<'a> {
    #[must_use]
    pub fn build(
        design: &'a Design,
        cell_refs_topo: &[CellRef<'a>],
        cell_id_map: &AHashMap<usize, CellIndex>,
        cells_topo: &[CellWrapper<'a>],
    ) -> Self {
        let (fanin_map, fanout_map) =
            Self::build_fanin_fanout_maps(design, cell_refs_topo, cell_id_map);

        // Precompute fanout and fanin sets
        let fanout_sets = Self::precompute_sets(&fanout_map);
        let fanin_sets = Self::precompute_sets(&fanin_map);

        // Precompute intersection of fanout of fanin
        let intersect_fanout_of_fanin =
            Self::precompute_intersect_fanout_of_fanin(&fanin_map, &fanout_sets);

        // Precompute CellWrapper sets for zero-cost queries
        let fanout_cell_sets = Self::precompute_cell_sets(&fanout_sets, cells_topo);
        let fanin_cell_sets = Self::precompute_cell_sets(&fanin_sets, cells_topo);
        let intersect_fanout_of_fanin_cells =
            Self::precompute_cell_sets(&intersect_fanout_of_fanin, cells_topo);

        Self {
            fanin_map,
            fanout_map,
            fanout_sets,
            fanin_sets,
            intersect_fanout_of_fanin,
            fanout_cell_sets,
            fanin_cell_sets,
            intersect_fanout_of_fanin_cells,
        }
    }

    fn build_fanin_fanout_maps(
        design: &'a Design,
        cell_refs_topo: &[CellRef<'a>],
        cell_id_map: &AHashMap<usize, CellIndex>,
    ) -> (FaninMap, FanoutMap) {
        let mut fanout_map: FanoutMap = AHashMap::new();
        let mut fanin_map: FaninMap = AHashMap::new();

        for sink_ref in cell_refs_topo.iter().copied() {
            let sink_wrapper: CellWrapper<'a> = sink_ref.into();
            sink_ref.visit(|net| {
                if let Ok((source_ref, source_pin_idx)) = design.find_cell(net) {
                    let source_ref_idx = *cell_id_map
                        .get(&source_ref.debug_index())
                        .expect("Source cell should be in map");

                    let sink_wrapper_idx = *cell_id_map
                        .get(&sink_wrapper.debug_index())
                        .expect("Sink cell should be in map");

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

    fn precompute_sets(
        map: &AHashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> AHashMap<CellIndex, AHashSet<CellIndex>> {
        map.iter()
            .map(|(idx, vec)| {
                let set = vec.iter().map(|(cell_idx, _)| *cell_idx).collect();
                (*idx, set)
            })
            .collect()
    }

    fn precompute_intersect_fanout_of_fanin(
        fanin_map: &FaninMap,
        fanout_sets: &AHashMap<CellIndex, AHashSet<CellIndex>>,
    ) -> AHashMap<CellIndex, AHashSet<CellIndex>> {
        fanin_map
            .iter()
            .filter_map(|(cell_idx, fanin_list)| {
                if fanin_list.is_empty() {
                    return None;
                }

                let fanout_sets_of_fanins: Vec<&AHashSet<CellIndex>> = fanin_list
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

    fn precompute_cell_sets(
        index_sets: &AHashMap<CellIndex, AHashSet<CellIndex>>,
        cells_topo: &[CellWrapper<'a>],
    ) -> AHashMap<CellIndex, AHashSet<CellWrapper<'a>>> {
        index_sets
            .iter()
            .map(|(cell_idx, index_set)| {
                let cell_set = index_set
                    .iter()
                    .map(|idx| cells_topo[idx.index()].clone())
                    .collect();
                (*cell_idx, cell_set)
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
    pub fn fanout_indices_set(&self, cell_idx: CellIndex) -> &AHashSet<CellIndex> {
        self.fanout_sets.get(&cell_idx).unwrap_or_else(|| {
            static EMPTY: std::sync::OnceLock<AHashSet<CellIndex>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(AHashSet::new)
        })
    }

    #[must_use]
    pub fn fanin_indices_set(&self, cell_idx: CellIndex) -> &AHashSet<CellIndex> {
        self.fanin_sets.get(&cell_idx).unwrap_or_else(|| {
            static EMPTY: std::sync::OnceLock<AHashSet<CellIndex>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(AHashSet::new)
        })
    }

    #[must_use]
    pub fn get_intersect_fanout_of_fanin_indices(
        &self,
        cell_idx: CellIndex,
    ) -> &AHashSet<CellIndex> {
        self.intersect_fanout_of_fanin
            .get(&cell_idx)
            .unwrap_or_else(|| {
                static EMPTY: std::sync::OnceLock<AHashSet<CellIndex>> = std::sync::OnceLock::new();
                EMPTY.get_or_init(AHashSet::new)
            })
    }

    #[must_use]
    pub const fn fanin_map(&self) -> &AHashMap<CellIndex, Vec<(CellIndex, usize)>> {
        &self.fanin_map
    }

    #[must_use]
    pub const fn fanout_map(&self) -> &AHashMap<CellIndex, Vec<(CellIndex, usize)>> {
        &self.fanout_map
    }

    #[must_use]
    pub fn fanout_cell_set(&self, cell_idx: CellIndex) -> Option<&AHashSet<CellWrapper<'a>>> {
        self.fanout_cell_sets.get(&cell_idx)
    }

    #[must_use]
    pub fn fanin_cell_set(&self, cell_idx: CellIndex) -> Option<&AHashSet<CellWrapper<'a>>> {
        self.fanin_cell_sets.get(&cell_idx)
    }

    #[must_use]
    pub fn intersect_fanout_of_fanin_cell_set(
        &self,
        cell_idx: CellIndex,
    ) -> Option<&AHashSet<CellWrapper<'a>>> {
        self.intersect_fanout_of_fanin_cells.get(&cell_idx)
    }
}
