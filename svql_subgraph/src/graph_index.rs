use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::{Cell, CellRef, Design};

use crate::cell::{CellKind, CellWrapper};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellIndex {
    idx: usize,
}

impl CellIndex {
    pub fn new(idx: usize) -> Self {
        CellIndex { idx }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.idx
    }
}

#[derive(Clone, Debug)]
pub struct GraphIndex<'a> {
    /// Nodes of haystack in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    cell_id_map: HashMap<usize, CellIndex>,

    // CellType map
    cell_type_map: HashMap<CellKind, Vec<CellIndex>>,

    // Input Fanout map
    fanin_map: HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    fanout_map: HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    clean_fanout_map: HashMap<CellIndex, HashSet<CellIndex>>,
    clean_fanin_map: HashMap<CellIndex, HashSet<CellIndex>>,

    // io fanin/out
    input_fanout_by_name: HashMap<String, Vec<(CellIndex, usize)>>,
    output_fanin_by_name: HashMap<String, Vec<(CellIndex, usize)>>,
}

impl<'a> GraphIndex<'a> {
    pub fn build(haystack: &'a Design) -> Self {
        // 0: Preparing a vector of cell references in topological order
        let cell_refs_topo = Self::build_cell_refs_topo(haystack);

        // 1: Building cells_topo
        let cells_topo = Self::build_cells_topo(&cell_refs_topo);

        let cell_id_map: HashMap<usize, CellIndex> = cells_topo
            .iter()
            .enumerate()
            .map(|(idx, cell)| (cell.debug_index(), CellIndex::new(idx)))
            .collect();

        // 2: Building cell_type_map
        let cell_type_map = Self::build_cell_type_map(&cell_refs_topo);

        // Building fanin/fanout_map
        let (fanin_map, fanout_map) =
            Self::build_fanin_fanout_maps(haystack, &cell_refs_topo, &cell_id_map);

        let clean_fanout_map = Self::build_clean_fanout_fanin_map(&fanout_map);
        let clean_fanin_map = Self::build_clean_fanout_fanin_map(&fanin_map);

        // #############
        // Building I/O Maps
        let input_fanout_by_name = Self::build_input_fanout_by_name(&cells_topo, &fanout_map);
        let output_fanin_by_name = Self::build_output_fanin_by_name(&cells_topo, &fanin_map);

        GraphIndex {
            cells_topo,

            cell_id_map,

            cell_type_map,
            fanout_map,
            fanin_map,
            //
            input_fanout_by_name,
            output_fanin_by_name,
            //
            clean_fanout_map,
            clean_fanin_map,
        }
    }

    fn build_cell_refs_topo(haystack: &'a Design) -> Vec<CellRef<'a>> {
        let cell_refs_topo: Vec<CellRef<'a>> = haystack
            .iter_cells_topo()
            .rev()
            .filter(|cell_ref| {
                let node_type = CellKind::from(cell_ref.get().as_ref());
                !matches!(node_type, CellKind::Name)
            })
            // .map(|cell_ref| cell_ref.get())
            .collect();
        cell_refs_topo
    }

    fn build_cells_topo(cell_refs_topo: &[CellRef<'a>]) -> Vec<CellWrapper<'a>> {
        let cell_refs_topo: Vec<CellWrapper<'a>> = cell_refs_topo
            .iter()
            .cloned()
            .map(|cell_ref| cell_ref.into())
            .collect();
        cell_refs_topo
    }

    fn build_cell_type_map(cell_refs_topo: &[CellRef<'a>]) -> HashMap<CellKind, Vec<CellIndex>> {
        let mut cell_type_map: HashMap<CellKind, Vec<CellIndex>> = HashMap::new();
        for (idx, cell) in cell_refs_topo.iter().enumerate() {
            let node_type = CellKind::from(cell.get().as_ref());
            cell_type_map
                .entry(node_type)
                .or_default()
                .push(CellIndex::new(idx));
        }
        cell_type_map
    }

    fn build_input_by_name(cells_topo: &[CellWrapper<'a>]) -> HashMap<&'a str, CellWrapper<'a>> {
        let input_by_name: HashMap<&'a str, CellWrapper<'a>> = cells_topo
            .iter()
            .filter_map(|c| {
                if matches!(c.cell_type(), CellKind::Input) {
                    let input_name: &'a str =
                        c.input_name().expect("Input cell should have a name");
                    Some((input_name, c.clone()))
                } else {
                    None
                }
            })
            .collect();
        input_by_name
    }

    fn build_output_fanin_by_name(
        cells_topo: &[CellWrapper<'a>],
        fanin_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<String, Vec<(CellIndex, usize)>> {
        let output_fanin_by_name: HashMap<String, Vec<(CellIndex, usize)>> = fanin_map
            .iter()
            .filter_map(|(cell_wrapper, value)| {
                if let Cell::Output(name, _) = cells_topo[cell_wrapper.idx].get() {
                    Some((name.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        output_fanin_by_name
    }

    fn build_input_fanout_by_name(
        cells_topo: &[CellWrapper<'a>],
        fanin_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<String, Vec<(CellIndex, usize)>> {
        let input_fanout_by_name: HashMap<String, Vec<(CellIndex, usize)>> = fanin_map
            .iter()
            .filter_map(|(cell_wrapper, value)| {
                if let Cell::Input(name, _) = cells_topo[cell_wrapper.idx].get() {
                    Some((name.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        input_fanout_by_name
    }

    fn build_fanin_fanout_maps(
        haystack: &'a Design,
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
                if let Ok((source_ref, source_pin_idx)) = haystack.find_cell(net) {
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

    fn build_clean_fanout_fanin_map(
        fanout_fanin_map: &HashMap<CellIndex, Vec<(CellIndex, usize)>>,
    ) -> HashMap<CellIndex, HashSet<CellIndex>> {
        let mut clean_fanout_fanin_map: HashMap<CellIndex, HashSet<CellIndex>> = HashMap::new();
        for (key, vec) in fanout_fanin_map.iter() {
            let set: HashSet<CellIndex> = vec.iter().map(|(c, _)| *c).collect();
            clean_fanout_fanin_map.insert(*key, set);
        }
        clean_fanout_fanin_map
    }

    pub fn cells_of_type(&self, node_type: CellKind) -> Vec<CellWrapper<'a>> {
        self.cell_type_map
            .get(&node_type)
            .map(|v| {
                v.iter()
                    .map(|idx| self.cells_topo[idx.idx].clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or(vec![])
    }

    pub fn topo_cells(&self) -> &[CellWrapper<'a>] {
        self.cells_topo.as_slice()
    }

    pub fn get_input_fanout_by_name(&self) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        self.input_fanout_by_name
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter()
                        .map(|(c, pin)| (self.cells_topo[c.idx].clone(), *pin))
                        .collect(),
                )
            })
            .collect()
    }

    pub fn get_output_fanin_by_name(&self) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        self.output_fanin_by_name
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter()
                        .map(|(c, pin)| (self.cells_topo[c.idx].clone(), *pin))
                        .collect(),
                )
            })
            .collect()
    }

    pub fn fanout_set(&self, cell: &CellWrapper<'a>) -> Option<HashSet<CellWrapper<'a>>> {
        let idx = self
            .cell_id_map
            .get(&cell.debug_index())
            .expect("Cell should be in map");
        self.clean_fanout_map.get(idx).map(|v| {
            v.iter()
                .map(|idx| self.cells_topo[idx.idx].clone())
                .collect()
        })
    }
    pub fn fanin_set(&self, cell: &CellWrapper<'a>) -> Option<HashSet<CellWrapper<'a>>> {
        let idx = self
            .cell_id_map
            .get(&cell.debug_index())
            .expect("Cell should be in map");
        self.clean_fanin_map.get(idx).map(|v| {
            v.iter()
                .map(|idx| self.cells_topo[idx.idx].clone())
                .collect()
        })
    }

    pub fn fanout_with_ports(
        &self,
        cell: &CellWrapper<'a>,
    ) -> Option<Vec<(CellWrapper<'a>, usize)>> {
        let idx = self
            .cell_id_map
            .get(&cell.debug_index())
            .expect("Cell should be in map");
        self.fanout_map.get(idx).map(|v| {
            v.iter()
                .map(|(idx, pin)| (self.cells_topo[idx.idx].clone(), *pin))
                .collect()
        })
    }
    pub fn fanin_with_ports(
        &self,
        cell: &CellWrapper<'a>,
    ) -> Option<Vec<(CellWrapper<'a>, usize)>> {
        let idx = self
            .cell_id_map
            .get(&cell.debug_index())
            .expect("Cell should be in map");
        self.fanin_map.get(idx).map(|v| {
            v.iter()
                .map(|(idx, pin)| (self.cells_topo[idx.idx].clone(), *pin))
                .collect()
        })
    }

    pub fn get_intersect_fanout_of_fanin(
        &self,
        cell: &CellWrapper<'a>,
    ) -> HashSet<CellWrapper<'a>> {
        let mut fanin: Vec<CellWrapper<'a>> = self
            .fanin_set(cell)
            .map(|s| s.clone().into_iter().collect())
            .unwrap_or_default();

        let first_fanin = fanin.remove(0);
        let initial_fanout = self
            .fanout_set(&first_fanin)
            .map(|s| s.clone().into_iter().collect())
            .unwrap_or_default();

        let fanout_of_fanin: HashSet<CellWrapper<'a>> = fanin
            .iter()
            .map(|c| {
                self.fanout_set(c)
                    .map(|s| s.clone().into_iter().collect())
                    .unwrap_or_default()
            })
            .fold(initial_fanout, |acc: HashSet<CellWrapper<'a>>, hs| {
                acc.intersection(&hs).cloned().collect()
            });

        fanout_of_fanin
    }
}
