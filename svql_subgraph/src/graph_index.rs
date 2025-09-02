use std::collections::HashMap;

use prjunnamed_netlist::{CellRef, Design};

use crate::{
    Timer,
    cell::{CellType, CellWrapper},
};

#[derive(Clone, Debug)]
pub(super) struct GraphIndex<'a> {
    /// Nodes of design in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    cell_type_map: HashMap<CellType, Vec<CellWrapper<'a>>>,

    fanout_map: HashMap<CellWrapper<'a>, Vec<CellWrapper<'a>>>,

    // ##########################################
    input_by_name: HashMap<&'a str, CellWrapper<'a>>,
    output_by_name: HashMap<&'a str, CellWrapper<'a>>,
}

impl<'a> GraphIndex<'a> {
    pub(super) fn build(design: &'a Design) -> Self {
        let _t = Timer::new("GraphIndex::build");

        // 0: Preparing a vector of cell references in topological order
        let cell_refs_topo = Self::build_cell_refs_topo(design);

        // 1: Building cells_topo
        let cells_topo = Self::build_cells_topo(&cell_refs_topo);

        // 2: Building cell_type_map
        let cell_type_map = Self::build_cell_type_map(&cell_refs_topo);

        // 3: Building fanout_map
        let fanout_map = Self::build_fanout_map(design, &cell_refs_topo);

        // #############
        // Building I/O Maps
        let input_by_name = Self::build_input_by_name(&cells_topo);
        let output_by_name = Self::build_output_by_name(&cells_topo);

        GraphIndex {
            cells_topo,
            cell_type_map,
            fanout_map,
            //
            input_by_name,
            output_by_name,
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

    fn build_fanout_map(
        design: &'a Design,
        cell_refs_topo: &[CellRef<'a>],
    ) -> HashMap<CellWrapper<'a>, Vec<CellWrapper<'a>>> {
        let mut fanout_map: HashMap<CellWrapper<'a>, Vec<CellWrapper<'a>>> = HashMap::new();
        for sink_ref in cell_refs_topo.iter().cloned() {
            let sink_wrapper: CellWrapper<'a> = sink_ref.into();
            sink_ref.visit(|net| {
                if let Ok((source_ref, _source_pin_idx)) = design.find_cell(net) {
                    let driver_wrapper: CellWrapper<'a> = source_ref.into();
                    fanout_map
                        .entry(driver_wrapper)
                        .or_default()
                        .push(sink_wrapper.clone());
                }
            });
        }
        fanout_map
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

    fn build_output_by_name(cells_topo: &[CellWrapper<'a>]) -> HashMap<&'a str, CellWrapper<'a>> {
        let output_by_name: HashMap<&'a str, CellWrapper<'a>> = cells_topo
            .iter()
            .filter_map(|c| {
                if matches!(c.cell_type(), CellType::Output) {
                    let output_name: &'a str =
                        c.output_name().expect("Output cell should have a name");
                    Some((output_name, c.clone()))
                } else {
                    None
                }
            })
            .collect();
        output_by_name
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

    pub fn get_output_by_name(&self) -> &HashMap<&'a str, CellWrapper<'a>> {
        &self.output_by_name
    }

    // pub(super) fn get_fanouts(&self, node: CellRef<'a>) -> &[(CellRef<'a>, usize)] {
    //     let _t = Timer::new("GraphIndex::get_fanouts");
    //     let slice = self
    //         .reverse_node_lookup
    //         .get(&node)
    //         .map(|v| v.as_slice())
    //         .unwrap_or(&[]);
    //     slice
    // }

    // pub(super) fn get_node_sources(&self, node: CellRef<'a>) -> &[NodeSource<'a>] {
    //     let _t = Timer::new("GraphIndex::get_node_sources");
    //     self.node_sources
    //         .get(&node)
    //         .map(|v| v.as_slice())
    //         .unwrap_or(&[])
    // }

    // pub(super) fn get_input_name(&self, node: CellRef<'a>) -> Option<&'a str> {
    //     let _t = Timer::new("GraphIndex::get_input_name");
    //     match node.get() {
    //         std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
    //         _ => None,
    //     }
    // }

    // pub(super) fn get_output_name(&self, node: CellRef<'a>) -> Option<&'a str> {
    //     let _t = Timer::new("GraphIndex::get_output_name");
    //     match node.get() {
    //         std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
    //         _ => None,
    //     }
    // }

    // pub(super) fn node_summary(&self, node: CellRef<'a>) -> String {
    //     let _t = Timer::new("GraphIndex::node_summary");
    //     let node_type = CellType::from(node.get().as_ref());
    //     let iname = self.get_input_name(node).unwrap_or("");
    //     let oname = self.get_output_name(node).unwrap_or("");
    //     let n = if !iname.is_empty() { iname } else { oname };
    //     if n.is_empty() {
    //         format!("#{} {:?}", node.debug_index(), node_type)
    //     } else {
    //         format!("#{} {:?}({})", node.debug_index(), node_type, n)
    //     }
    // }

    // /// Named fan‑in accessor: returns a named port -> sources map (by bit).
    // pub(super) fn get_node_fanin_named(&self, node: CellRef<'a>) -> &NodeFanin<'a> {
    //     // Constructed for all nodes in build(); unwrap is safe.
    //     self.fanin_named
    //         .get(&node)
    //         .expect("missing NodeFanin for node")
    // }

    // /// True if `driver` has any fanout edge to `sink` (any input pin).
    // pub(super) fn has_fanout_to(&self, driver: CellRef<'a>, sink: CellRef<'a>) -> bool {
    //     let _t = Timer::new("GraphIndex::has_fanout_to");
    //     self.fanout_map
    //         .get(&driver)
    //         .and_then(|m| m.get(&sink))
    //         .is_some()
    // }

    // /// True if `driver` feeds `sink` specifically at `pin_idx`.
    // pub(super) fn has_fanout_to_pin(
    //     &self,
    //     driver: CellRef<'a>,
    //     sink: CellRef<'a>,
    //     pin_idx: usize,
    // ) -> bool {
    //     let _t = Timer::new("GraphIndex::has_fanout_to_pin");
    //     self.fanout_map
    //         .get(&driver)
    //         .and_then(|m| m.get(&sink))
    //         .is_some_and(|pins| pins.contains(&pin_idx))
    // }

    // /// The unique driver of a given sink input pin, if any (Gate/Io sources only).
    // pub(super) fn driver_of_sink_pin(
    //     &self,
    //     sink: CellRef<'a>,
    //     pin_idx: usize,
    // ) -> Option<CellRef<'a>> {
    //     let _t = Timer::new("GraphIndex::driver_of_sink_pin");
    //     let src = self.get_node_sources(sink).get(pin_idx)?;
    //     match src {
    //         NodeSource::Gate(c, _) | NodeSource::Io(c, _) => Some(*c),
    //         NodeSource::Const(_) => None,
    //     }
    // }

    // /// All drivers of a sink across all pins (Gate/Io only), duplicates removed.
    // pub(super) fn drivers_of_sink_all_pins(&self, sink: CellRef<'a>) -> Vec<CellRef<'a>> {
    //     let _t = Timer::new("GraphIndex::drivers_of_sink_all_pins");
    //     let mut out: Vec<CellRef<'a>> = self
    //         .get_node_sources(sink)
    //         .iter()
    //         .filter_map(|src| match src {
    //             NodeSource::Gate(c, _) | NodeSource::Io(c, _) => Some(*c),
    //             NodeSource::Const(_) => None,
    //         })
    //         .collect();

    //     out.sort_by_key(|c| c.debug_index());
    //     out.dedup();
    //     out
    // }
}
