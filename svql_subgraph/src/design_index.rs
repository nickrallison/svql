use std::{collections::HashMap, vec};

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

    fanout_map: HashMap<CellWrapper<'a>, Vec<CellWrapper<'a>>>,

    // ##########################################
    input_by_name: HashMap<&'a str, CellWrapper<'a>>,
    output_by_name: HashMap<&'a str, CellWrapper<'a>>,

    // Input Fanout map
    input_fanin_map: HashMap<String, Vec<(CellWrapper<'a>, usize)>>,
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

        // #############
        let input_fanin_map: HashMap<String, Vec<(CellWrapper<'a>, usize)>> =
            Self::build_input_fanin_map(&cells_topo, design);

        DesignIndex {
            cells_topo,
            cell_type_map,
            fanout_map,
            //
            input_by_name,
            output_by_name,
            //
            input_fanin_map,
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

    fn build_input_fanin_map(
        cells_topo: &[CellWrapper<'a>],
        design: &'a Design,
    ) -> HashMap<String, Vec<(CellWrapper<'a>, usize)>> {
        let mut input_fanin_map: HashMap<String, Vec<(CellWrapper<'a>, usize)>> = HashMap::new();
        for cell in cells_topo {
            let input_fanin = get_cell_index_fanin(cell.get(), design);
            for (input_name, input_idx) in input_fanin {
                input_fanin_map
                    .entry(input_name)
                    .or_default()
                    .push((cell.clone(), input_idx));
            }
        }
        input_fanin_map
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

pub fn get_cell_index_fanin<'a>(
    cell: &Cell,
    design: &prjunnamed_netlist::Design,
) -> Vec<(String, usize)> {
    use prjunnamed_netlist::Cell::*;
    match cell {
        Buf(p_value) => get_value_index_fanin(p_value, design),
        Not(p_value) => get_value_index_fanin(p_value, design),
        And(p_a_value, p_b_value) => {
            let mut a_matches = get_value_index_fanin(p_a_value, design);
            let b_matches = get_value_index_fanin(p_b_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        Or(p_a_value, p_b_value) => {
            let mut a_matches = get_value_index_fanin(p_a_value, design);
            let b_matches = get_value_index_fanin(p_b_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        Xor(p_a_value, p_b_value) => {
            let mut a_matches = get_value_index_fanin(p_a_value, design);
            let b_matches = get_value_index_fanin(p_b_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        Mux(p_a_net, p_b_value, p_c_value) => {
            let mut a_matches = get_net_index_fanin(p_a_net, design);
            let b_matches = get_value_index_fanin(p_b_value, design);
            let c_matches = get_value_index_fanin(p_c_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches.extend(c_matches.into_iter());
            a_matches
        }
        Adc(p_a_value, p_b_value, p_ci_net) => {
            let mut a_matches = get_value_index_fanin(p_a_value, design);
            let b_matches = get_value_index_fanin(p_b_value, design);
            let ci_matches = get_net_index_fanin(p_ci_net, design);
            a_matches.extend(b_matches.into_iter());
            a_matches.extend(ci_matches.into_iter());
            a_matches
        }
        Aig(pa_control_net, pb_control_net) => {
            let mut a_matches = get_control_net_index_fanin(pa_control_net, design);
            let b_matches = get_control_net_index_fanin(pb_control_net, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        Eq(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        ULt(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        SLt(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        Shl(pa_value, pb_value, _pc_u32) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        UShr(pa_value, pb_value, _pc_u32) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        SShr(pa_value, pb_value, _pc_u32) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        XShr(pa_value, pb_value, _pc_u32) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        Mul(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        UDiv(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        UMod(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        SDivTrunc(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        SDivFloor(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        SModTrunc(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        SModFloor(pa_value, pb_value) => {
            let mut a_matches = get_value_index_fanin(pa_value, design);
            let b_matches = get_value_index_fanin(pb_value, design);
            a_matches.extend(b_matches.into_iter());
            a_matches
        }
        Match(_p_match_cell) => {
            todo!("Make Function to match match cells")
        }
        Assign(_p_assign_cell) => {
            todo!("Make Function to match assign cells")
        }
        Dff(p_dff_cell) => {
            let mut data_matches = get_value_index_fanin(&p_dff_cell.data, design);
            let clock_matches = get_control_net_index_fanin(&p_dff_cell.clock, design);
            let clear_matches = get_control_net_index_fanin(&p_dff_cell.clear, design);
            let reset_matches = get_control_net_index_fanin(&p_dff_cell.reset, design);
            let enable_matches = get_control_net_index_fanin(&p_dff_cell.enable, design);
            data_matches.extend(clock_matches.into_iter());
            data_matches.extend(clear_matches.into_iter());
            data_matches.extend(reset_matches.into_iter());
            data_matches.extend(enable_matches.into_iter());
            data_matches
        }
        Memory(_p_memory_cell) => {
            todo!("Make Function to match memory cells")
        }
        // (IoBuf(pi), IoBuf(di)) => todo!(),
        // (Target(pt), Target(dt)) => todo!(),
        // (Other(po), Other(do_)) => todo!(),
        Input(_p_name, _p_width) => return vec![],
        // (Output(pn, pv), Output(dn, dv)) => todo!(),
        // (Name(pn, pv), Name(dn, dv)) => todo!(),
        // (Debug(pn, pv), Debug(dn, dv)) => todo!(),
        _ => vec![],
    }
}

fn get_value_index_fanin<'a>(
    value: &prjunnamed_netlist::Value,
    design: &prjunnamed_netlist::Design,
) -> Vec<(String, usize)> {
    match &value.0 {
        prjunnamed_netlist::ValueRepr::None => vec![],
        prjunnamed_netlist::ValueRepr::Some(net) => get_net_index_fanin(&net, design),
        prjunnamed_netlist::ValueRepr::Many(nets) => nets
            .iter()
            .flat_map(|net| get_net_index_fanin(&net, design))
            .collect(),
    }
}
fn get_net_index_fanin<'a>(
    net: &prjunnamed_netlist::Net,
    design: &prjunnamed_netlist::Design,
) -> Vec<(String, usize)> {
    if let Ok((cell_ref, id)) = design.find_cell(*net) {
        let cell_wrapper: CellWrapper = cell_ref.into();
        if let Some(name) = cell_wrapper.input_name() {
            return vec![(name.to_string(), id)];
        }
    }
    vec![]
}

fn get_control_net_index_fanin<'a>(
    pa_control_net: &prjunnamed_netlist::ControlNet,
    design: &prjunnamed_netlist::Design,
) -> Vec<(String, usize)> {
    match pa_control_net {
        prjunnamed_netlist::ControlNet::Pos(net) => get_net_index_fanin(net, design),
        prjunnamed_netlist::ControlNet::Neg(net) => get_net_index_fanin(net, design),
    }
}
