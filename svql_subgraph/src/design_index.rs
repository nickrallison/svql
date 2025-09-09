use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::{Cell, CellRef, Design};

use crate::{
    Timer,
    cell::{CellType, CellWrapper},
};

#[derive(Clone, Debug)]
pub struct DesignIndex<'a> {
    /// Nodes of design in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    cell_type_map: HashMap<CellType, Vec<CellWrapper<'a>>>,

    // ##########################################
    input_by_name: HashMap<&'a str, CellWrapper<'a>>,
    output_by_name: HashMap<String, Vec<(CellWrapper<'a>, usize)>>,

    // Input Fanout map
    fanin_map: HashMap<usize, Vec<(CellWrapper<'a>, usize)>>,
    fanout_map: HashMap<usize, Vec<(CellWrapper<'a>, usize)>>,

    //
    clean_fanout_map: HashMap<usize, HashSet<CellWrapper<'a>>>,
    clean_fanin_map: HashMap<usize, HashSet<CellWrapper<'a>>>,
}

impl<'a> DesignIndex<'a> {
    pub fn build(design: &'a Design) -> Self {
        let _t = Timer::new("GraphIndex::build");

        // 0: Preparing a vector of cell references in topological order
        let cell_refs_topo = Self::build_cell_refs_topo(design);

        // 1: Building cells_topo
        let cells_topo = Self::build_cells_topo(&cell_refs_topo);

        // 2: Building cell_type_map
        let cell_type_map = Self::build_cell_type_map(&cell_refs_topo);

        // #############
        // Building I/O Maps
        let input_by_name = Self::build_input_by_name(&cells_topo);
        let output_by_name = Self::build_output_by_name(&cells_topo, design);

        // Building fanin/fanout_map
        let (fanin_map, fanout_map) = Self::build_fanin_fanout_maps(design, &cell_refs_topo);

        let clean_fanout_map = Self::build_clean_fanout_fanin_map(&fanout_map);
        let clean_fanin_map = Self::build_clean_fanout_fanin_map(&fanin_map);

        DesignIndex {
            cells_topo,
            cell_type_map,
            fanout_map,
            fanin_map,
            //
            input_by_name,
            output_by_name,
            //
            clean_fanout_map,
            clean_fanin_map,
        }
    }

    fn build_cell_refs_topo(design: &'a Design) -> Vec<CellRef<'a>> {
        let cell_refs_topo: Vec<CellRef<'a>> = design
            .iter_cells_topo()
            .rev()
            .filter(|cell_ref| {
                let node_type = CellType::from(cell_ref.get().as_ref());
                !matches!(node_type, CellType::Name)
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

    fn build_cell_type_map(
        cell_refs_topo: &[CellRef<'a>],
    ) -> HashMap<CellType, Vec<CellWrapper<'a>>> {
        let mut cell_type_map: HashMap<CellType, Vec<CellWrapper<'a>>> = HashMap::new();
        for cell in cell_refs_topo {
            let node_type = CellType::from(cell.get().as_ref());
            cell_type_map
                .entry(node_type)
                .or_default()
                .push(cell.clone().into());
        }
        cell_type_map
    }

    fn build_input_by_name(cells_topo: &[CellWrapper<'a>]) -> HashMap<&'a str, CellWrapper<'a>> {
        let input_by_name: HashMap<&'a str, CellWrapper<'a>> = cells_topo
            .iter()
            .filter_map(|c| {
                if matches!(c.cell_type(), CellType::Input) {
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

    fn build_output_by_name(
        cells_topo: &[CellWrapper<'a>],
        design: &'a Design,
    ) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        let output_by_name: HashMap<String, Vec<(CellWrapper<'a>, usize)>> = cells_topo
            .iter()
            .filter_map(|c| {
                if let Cell::Output(name, value) = c.get() {
                    let mut visited: Vec<(CellWrapper<'a>, usize)> = Vec::new();
                    value.visit(|net| {
                        let cell = design.find_cell(net);

                        if let Ok((source_ref, id)) = cell {
                            let source_wrapper: CellWrapper<'a> = source_ref.into();
                            if !visited.contains(&(source_wrapper.clone(), id)) {
                                visited.push((source_wrapper, id));
                            }
                        }
                    });
                    Some((name.to_string(), visited))
                } else {
                    None
                }
            })
            .collect();
        output_by_name
    }

    fn build_fanin_fanout_maps(
        design: &'a Design,
        cell_refs_topo: &[CellRef<'a>],
    ) -> (
        HashMap<usize, Vec<(CellWrapper<'a>, usize)>>,
        HashMap<usize, Vec<(CellWrapper<'a>, usize)>>,
    ) {
        let mut fanout_map: HashMap<usize, Vec<(CellWrapper<'a>, usize)>> = HashMap::new();
        let mut fanin_map: HashMap<usize, Vec<(CellWrapper<'a>, usize)>> = HashMap::new();
        for sink_ref in cell_refs_topo.iter().cloned() {
            let sink_wrapper: CellWrapper<'a> = sink_ref.into();
            sink_ref.visit(|net| {
                if let Ok((source_ref, source_pin_idx)) = design.find_cell(net) {
                    fanout_map
                        .entry(source_ref.debug_index())
                        .or_default()
                        .push((sink_wrapper.clone(), source_pin_idx));
                    fanin_map
                        .entry(sink_wrapper.debug_index())
                        .or_default()
                        .push((source_ref.clone().into(), source_pin_idx));
                }
            });
        }
        (fanin_map, fanout_map)
    }

    fn build_clean_fanout_fanin_map(
        fanout_fanin_map: &HashMap<usize, Vec<(CellWrapper<'a>, usize)>>,
    ) -> HashMap<usize, HashSet<CellWrapper<'a>>> {
        let mut clean_fanout_fanin_map: HashMap<usize, HashSet<CellWrapper<'a>>> = HashMap::new();
        for (key, vec) in fanout_fanin_map.iter() {
            let set: HashSet<CellWrapper<'a>> = vec.iter().map(|(c, _)| c.clone()).collect();
            clean_fanout_fanin_map.insert(*key, set);
        }
        clean_fanout_fanin_map
    }

    pub fn get_by_type(&self, node_type: CellType) -> &[CellWrapper<'a>] {
        let _t = Timer::new("GraphIndex::get_by_type");
        self.cell_type_map
            .get(&node_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_cells_topo(&self) -> &[CellWrapper<'a>] {
        self.cells_topo.as_slice()
    }

    pub fn get_input_by_name(&self) -> &HashMap<&'a str, CellWrapper<'a>> {
        &self.input_by_name
    }

    pub fn get_output_by_name(&self) -> &HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        &self.output_by_name
    }

    pub fn get_fanout(&self, cell: &CellWrapper<'a>) -> Option<&HashSet<CellWrapper<'a>>> {
        self.clean_fanout_map.get(&cell.debug_index())
    }
    pub fn get_fanin(&self, cell: &CellWrapper<'a>) -> Option<&HashSet<CellWrapper<'a>>> {
        self.clean_fanin_map.get(&cell.debug_index())
    }

    pub fn get_fanout_raw(&self, cell: &CellWrapper<'a>) -> Option<&Vec<(CellWrapper<'a>, usize)>> {
        self.fanout_map.get(&cell.debug_index())
    }
    pub fn get_fanin_raw(&self, cell: &CellWrapper<'a>) -> Option<&Vec<(CellWrapper<'a>, usize)>> {
        self.fanin_map.get(&cell.debug_index())
    }

    pub fn get_intersect_fanout_of_fanin(
        &self,
        cell: &CellWrapper<'a>,
    ) -> HashSet<CellWrapper<'a>> {
        let mut fanin: Vec<CellWrapper<'a>> = self
            .get_fanin(cell)
            .map(|s| s.clone().into_iter().collect())
            .unwrap_or_default();

        let first_fanin = fanin.remove(0);
        let initial_fanout = self
            .get_fanout(&first_fanin)
            .map(|s| s.clone().into_iter().collect())
            .unwrap_or_default();

        let fanout_of_fanin: HashSet<CellWrapper<'a>> = fanin
            .iter()
            .map(|c| {
                self.get_fanout(c)
                    .map(|s| s.clone().into_iter().collect())
                    .unwrap_or_default()
            })
            .fold(initial_fanout, |acc: HashSet<CellWrapper<'a>>, hs| {
                acc.intersection(&hs).cloned().collect()
            });

        fanout_of_fanin
    }
}
