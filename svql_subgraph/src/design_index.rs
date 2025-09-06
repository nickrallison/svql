use std::collections::HashMap;

use prjunnamed_netlist::{CellRef, Design};

use crate::{
    Timer,
    cell::{CellType, CellWrapper},
};

#[derive(Clone, Debug)]
pub struct DesignIndex<'a> {
    /// Nodes of design in topological order (Name nodes filtered out)
    cells_topo: Vec<CellWrapper<'a>>,
    cell_type_map: HashMap<CellType, Vec<CellWrapper<'a>>>,

    fanout_map: HashMap<CellWrapper<'a>, Vec<CellWrapper<'a>>>,

    // ##########################################
    input_by_name: HashMap<&'a str, CellWrapper<'a>>,
    output_by_name: HashMap<&'a str, CellWrapper<'a>>,
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

        // 3: Building fanout_map
        let fanout_map = Self::build_fanout_map(design, &cell_refs_topo);

        // #############
        // Building I/O Maps
        let input_by_name = Self::build_input_by_name(&cells_topo);
        let output_by_name = Self::build_output_by_name(&cells_topo);

        DesignIndex {
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
}
