use crate::cell_mapping::CellMapping;
use crate::constraints::Constraint;
use crate::graph_index::GraphIndex;
use crate::{Timer, cell::CellWrapper};
use prjunnamed_netlist::{Cell, CellRef, Design, FlipFlop, Trit, Value, ValueRepr};
use svql_common::Config;
use tracing::{debug, trace};

pub(crate) struct ConnectivityConstraint<'a, 'p, 'd> {
    p_cell: CellWrapper<'p>,
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,

    pattern: &'p Design,
    design: &'d Design,

    config: &'a Config,
    mapping: CellMapping<'p, 'd>,
}

impl<'a, 'p, 'd> ConnectivityConstraint<'a, 'p, 'd> {
    pub(crate) fn new(
        p_cell: CellWrapper<'p>,
        pattern_index: &'a GraphIndex<'p>,
        design_index: &'a GraphIndex<'d>,
        pattern: &'p Design,
        design: &'d Design,
        config: &'a Config,
        mapping: CellMapping<'p, 'd>,
    ) -> Self {
        ConnectivityConstraint {
            p_cell,
            pattern_index,
            design_index,
            pattern,
            design,
            config,
            mapping,
        }
    }

    fn is_cell_connectivity_valid(&self, d_cell: CellWrapper<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::is_cell_connectivity_valid");
        self.validate_fanin_connections(d_cell)
    }

    fn validate_fanin_connections(&self, d_cell: CellWrapper<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::validate_fanin_connections");

        // let p_cell_cow = self.p_cell.get();
        // let d_cell_cow = d_cell.get();

        let p_cell: &Cell = self.p_cell.get();
        let d_cell: &Cell = d_cell.get();

        self.cells_match_fan_in(p_cell, d_cell)
    }

    // ####################################
    fn cells_match_fan_in(&self, pattern_cell: &Cell, design_cell: &Cell) -> bool {
        trace!(
            "Checking if cells match fan-in: {:?} and {:?}",
            pattern_cell, design_cell
        );
        use Cell::*;
        match (pattern_cell, design_cell) {
            (Buf(p_value), Buf(d_value)) => {
                let value_matches = self.values_match_fan_in(p_value, d_value);
                value_matches
            }
            (Not(p_value), Not(d_value)) => {
                let value_matches = self.values_match_fan_in(p_value, d_value);
                value_matches
            }
            (And(p_a_value, p_b_value), And(d_a_value, d_b_value)) => {
                let a_matches = self.values_match_fan_in(p_a_value, d_a_value);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value);
                let result_normal = a_matches && b_matches;

                // Commutative case
                let a_swapped = self.values_match_fan_in(p_a_value, d_b_value);
                let b_swapped = self.values_match_fan_in(p_b_value, d_a_value);
                let result_swapped = a_swapped && b_swapped;

                result_normal || result_swapped
            }
            (Or(p_a_value, p_b_value), Or(d_a_value, d_b_value)) => {
                let a_matches = self.values_match_fan_in(p_a_value, d_a_value);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value);
                let result_normal = a_matches && b_matches;

                // Commutative case
                let a_swapped = self.values_match_fan_in(p_a_value, d_b_value);
                let b_swapped = self.values_match_fan_in(p_b_value, d_a_value);
                let result_swapped = a_swapped && b_swapped;

                result_normal || result_swapped
            }
            (Xor(p_a_value, p_b_value), Xor(d_a_value, d_b_value)) => {
                let a_matches = self.values_match_fan_in(p_a_value, d_a_value);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value);
                let result_normal = a_matches && b_matches;

                // Commutative case
                let a_swapped = self.values_match_fan_in(p_a_value, d_b_value);
                let b_swapped = self.values_match_fan_in(p_b_value, d_a_value);
                let result_swapped = a_swapped && b_swapped;

                result_normal || result_swapped
            }
            (Mux(p_a_value, p_b_value, p_c_value), Mux(d_a_value, d_b_value, d_c_value)) => {
                let a_matches = self.nets_match_fan_in(p_a_value, d_a_value);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value);
                let c_matches = self.values_match_fan_in(p_c_value, d_c_value);
                let result = a_matches && b_matches && c_matches;

                match result {
                    true => trace!(
                        "Mux values match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                    false => trace!(
                        "Mux values do NOT match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                };
                result
            }
            (Adc(p_a_value, p_b_value, p_ci_net), Adc(d_a_value, d_b_value, d_ci_net)) => {
                let a_matches = self.values_match_fan_in(p_a_value, d_a_value);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value);
                let ci_matches = self.nets_match_fan_in(p_ci_net, d_ci_net);
                let result = a_matches && b_matches && ci_matches;

                match result {
                    true => trace!(
                        "Adc values match fan-in: a={:?}, b={:?}, ci={:?}",
                        a_matches, b_matches, ci_matches
                    ),
                    false => trace!(
                        "Adc values do NOT match fan-in: a={:?}, b={:?}, ci={:?}",
                        a_matches, b_matches, ci_matches
                    ),
                };
                result
            }
            (Aig(pa_control_net, pb_control_net), Aig(da_control_net, db_control_net)) => {
                let a_matches = self.control_nets_match_fan_in(pa_control_net, da_control_net);
                let b_matches = self.control_nets_match_fan_in(pb_control_net, db_control_net);
                let result = a_matches && b_matches;

                match result {
                    true => trace!(
                        "Aig values match fan-in: a={:?}, b={:?}",
                        a_matches, b_matches
                    ),
                    false => trace!(
                        "Aig values do NOT match fan-in: a={:?}, b={:?}",
                        a_matches, b_matches
                    ),
                };
                result
            }
            (Eq(pa_value, pb_value), Eq(da_value, db_value)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value);
                let b_matches = self.values_match_fan_in(pb_value, db_value);
                let result_normal = a_matches && b_matches;

                // Commutative case
                let a_swapped = self.values_match_fan_in(pa_value, db_value);
                let b_swapped = self.values_match_fan_in(pb_value, da_value);
                let result_swapped = a_swapped && b_swapped;

                result_normal || result_swapped
            }
            (ULt(pa_value, pb_value), ULt(da_value, db_value)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value);
                let b_matches = self.values_match_fan_in(pb_value, db_value);
                let result = a_matches && b_matches;

                match result {
                    true => trace!(
                        "ULt values match fan-in: a={:?}, b={:?}",
                        a_matches, b_matches
                    ),
                    false => trace!(
                        "ULt values do NOT match fan-in: a={:?}, b={:?}",
                        a_matches, b_matches
                    ),
                };
                result
            }
            (SLt(pa_value, pb_value), SLt(da_value, db_value)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value);
                let b_matches = self.values_match_fan_in(pb_value, db_value);
                let result = a_matches && b_matches;

                match result {
                    true => trace!(
                        "SLt values match fan-in: a={:?}, b={:?}",
                        a_matches, b_matches
                    ),
                    false => trace!(
                        "SLt values do NOT match fan-in: a={:?}, b={:?}",
                        a_matches, b_matches
                    ),
                };
                result
            }
            (Shl(pa_value, pb_value, pc_u32), Shl(da_value, db_value, dc_u32)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value);
                let b_matches = self.values_match_fan_in(pb_value, db_value);
                let c_matches = pc_u32 == dc_u32;
                let result = a_matches && b_matches && c_matches;

                match result {
                    true => trace!(
                        "Shl values match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                    false => trace!(
                        "Shl values do NOT match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                };
                result
            }
            (UShr(pa_value, pb_value, pc_u32), UShr(da_value, db_value, dc_u32)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value);
                let b_matches = self.values_match_fan_in(pb_value, db_value);
                let c_matches = pc_u32 == dc_u32;
                let result = a_matches && b_matches && c_matches;

                match result {
                    true => trace!(
                        "UShr values match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                    false => trace!(
                        "UShr values do NOT match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                };
                result
            }
            (SShr(pa_value, pb_value, pc_u32), SShr(da_value, db_value, dc_u32)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value);
                let b_matches = self.values_match_fan_in(pb_value, db_value);
                let c_matches = pc_u32 == dc_u32;
                let result = a_matches && b_matches && c_matches;

                match result {
                    true => trace!(
                        "SShr values match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                    false => trace!(
                        "SShr values do NOT match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                };
                result
            }
            (XShr(pa_value, pb_value, pc_u32), XShr(da_value, db_value, dc_u32)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value);
                let b_matches = self.values_match_fan_in(pb_value, db_value);
                let c_matches = pc_u32 == dc_u32;
                let result = a_matches && b_matches && c_matches;

                match result {
                    true => trace!(
                        "XShr values match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                    false => trace!(
                        "XShr values do NOT match fan-in: a={:?}, b={:?}, c={:?}",
                        a_matches, b_matches, c_matches
                    ),
                };
                result
            }
            (Mul(pa_value, pb_value), Mul(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value)
                    && self.values_match_fan_in(pb_value, db_value)
            }
            (UDiv(pa_value, pb_value), UDiv(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value)
                    && self.values_match_fan_in(pb_value, db_value)
            }
            (UMod(pa_value, pb_value), UMod(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value)
                    && self.values_match_fan_in(pb_value, db_value)
            }
            (SDivTrunc(pa_value, pb_value), SDivTrunc(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value)
                    && self.values_match_fan_in(pb_value, db_value)
            }
            (SDivFloor(pa_value, pb_value), SDivFloor(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value)
                    && self.values_match_fan_in(pb_value, db_value)
            }
            (SModTrunc(pa_value, pb_value), SModTrunc(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value)
                    && self.values_match_fan_in(pb_value, db_value)
            }
            (SModFloor(pa_value, pb_value), SModFloor(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value)
                    && self.values_match_fan_in(pb_value, db_value)
            }
            (Match(p_match_cell), Match(d_match_cell)) => {
                todo!("Make Function to match match cells")
            }
            (Assign(p_assign_cell), Assign(d_assign_cell)) => {
                todo!("Make Function to match assign cells")
            }
            (Dff(p_dff_cell), Dff(d_dff_cell)) => self.dffs_match_fan_in(p_dff_cell, d_dff_cell),
            (Memory(p_memory_cell), Memory(d_memory_cell)) => {
                todo!("Make Function to match memory cells")
            }
            // (IoBuf(pi), IoBuf(di)) => todo!(),
            // (Target(pt), Target(dt)) => todo!(),
            // (Other(po), Other(do_)) => todo!(),
            (Input(p_name, p_width), Input(d_name, d_width)) => {
                // panic!(
                //     "p_name: {p_name}, p_width: {p_width}, d_name: {d_name}, d_width: {d_width}"
                // );
                return true;
                todo!("decide how input cells should be matched for fan in")
            }
            (Input(p_name, p_width), d_cell) => {
                // panic!(
                //     "p_name: {p_name}, p_width: {p_width}, d_name: {d_name}, d_width: {d_width}"
                // );
                return true;
                todo!("decide how input cells should be matched for fan in")
            }
            // (Output(pn, pv), Output(dn, dv)) => todo!(),
            // (Name(pn, pv), Name(dn, dv)) => todo!(),
            // (Debug(pn, pv), Debug(dn, dv)) => todo!(),
            _ => false,
        }
    }

    fn values_match_fan_in(&self, pattern_value: &Value, design_value: &Value) -> bool {
        trace!(
            "Checking if values match fan-in: {:?} and {:?}",
            pattern_value, design_value
        );
        let pattern_value_repr: &ValueRepr = &pattern_value.0;
        let design_value_repr: &ValueRepr = &design_value.0;
        match (pattern_value_repr, design_value_repr) {
            (ValueRepr::None, ValueRepr::None) => true,
            (ValueRepr::Some(p_net), ValueRepr::Some(d_net)) => {
                self.nets_match_fan_in(&p_net, &d_net)
            }
            (ValueRepr::Many(p_nets), ValueRepr::Many(d_nets)) => {
                todo!(
                    "Use config to control how nets of different sizes match, then use nets_match_fan_in method"
                )
            }
            (ValueRepr::Some(p_net), ValueRepr::Many(d_nets)) => {
                let first_d_net = d_nets.first().unwrap();
                return self.nets_match_fan_in(&p_net, first_d_net);
                todo!("Should single pattern match against many design?")
            }
            _ => false,
        }
    }

    fn nets_match_fan_in(
        &self,
        pattern_net: &prjunnamed_netlist::Net,
        design_net: &prjunnamed_netlist::Net,
    ) -> bool {
        trace!(
            "Checking if nets match fan-in: {:?} and {:?}",
            pattern_net, design_net
        );
        let actual_fan_in_design_cell: Result<(CellRef<'d>, usize), Trit> =
            self.design.find_cell(*design_net);
        let fan_in_pattern_cell: Result<(CellRef<'p>, usize), Trit> =
            self.pattern.find_cell(*pattern_net);

        let (actual_fan_in_design_cell_ref, d_fan_in_idx, fan_in_pattern_cell_ref, p_fan_in_idx) =
            match (actual_fan_in_design_cell, fan_in_pattern_cell) {
                (Ok((d_fan_in_cell_ref, d_bit_idx)), Ok((p_fan_in_cell_ref, p_bit_idx))) => {
                    (d_fan_in_cell_ref, d_bit_idx, p_fan_in_cell_ref, p_bit_idx)
                }
                (Err(design_trit), Err(pattern_trit)) => return design_trit == pattern_trit,
                _ => return false,
            };

        let expected_fan_in_design_cell_opt =
            self.mapping.get_design_cell(fan_in_pattern_cell_ref.into());

        if expected_fan_in_design_cell_opt.is_none() {
            // Pattern fan-in cell not mapped yet, so we can't constrain it here.
            return true;
        }

        let expected_fan_in_design_cell_wrapper = expected_fan_in_design_cell_opt.unwrap();

        return expected_fan_in_design_cell_wrapper == actual_fan_in_design_cell_ref.into();

        todo!("How to handle expected idx");
    }

    fn control_nets_match_fan_in(
        &self,
        pattern_c_net: &prjunnamed_netlist::ControlNet,
        design_c_net: &prjunnamed_netlist::ControlNet,
    ) -> bool {
        trace!(
            "Checking if control nets match fan-in: {:?} and {:?}",
            pattern_c_net, design_c_net
        );
        match (pattern_c_net, design_c_net) {
            (
                prjunnamed_netlist::ControlNet::Pos(p_pos_net),
                prjunnamed_netlist::ControlNet::Pos(d_pos_net),
            ) => self.nets_match_fan_in(p_pos_net, d_pos_net),
            (
                prjunnamed_netlist::ControlNet::Neg(p_neg_net),
                prjunnamed_netlist::ControlNet::Neg(d_neg_net),
            ) => self.nets_match_fan_in(p_neg_net, d_neg_net),
            _ => false,
        }
    }

    fn const_match_fan_in(
        &self,
        pattern_const: &prjunnamed_netlist::Const,
        design_const: &prjunnamed_netlist::Const,
    ) -> bool {
        trace!(
            "Checking if consts match fan-in: {:?} and {:?}",
            pattern_const, design_const
        );
        let mut pattern_const_iter = pattern_const.clone().into_iter();
        let mut design_const_iter = design_const.clone().into_iter();

        while let (Some(p_t), Some(d_t)) = (pattern_const_iter.next(), design_const_iter.next()) {
            if p_t != d_t {
                return false;
            }
        }
        true
    }

    // #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    // pub struct FlipFlop {
    //     pub data: Value,
    //     /// The clock.  The active edge is rising if it is a [`ControlNet::Pos`], and falling if it is
    //     /// a [`ControlNet::Neg`].
    //     pub clock: ControlNet,
    //     /// Asynchronous reset.
    //     pub clear: ControlNet,
    //     /// Synchronous reset.
    //     pub reset: ControlNet,
    //     /// Clock enable.
    //     pub enable: ControlNet,
    //     /// If true, `reset` has priority over `enable`.  Otherwise, `enable` has priority over `reset`.
    //     pub reset_over_enable: bool,

    //     /// Must have the same width as `data`.
    //     pub clear_value: Const,
    //     /// Must have the same width as `data`.
    //     pub reset_value: Const,
    //     /// Must have the same width as `data`.
    //     pub init_value: Const,
    // }

    fn dffs_match_fan_in(&self, pattern_dff: &FlipFlop, design_dff: &FlipFlop) -> bool {
        trace!(
            "Checking if DFFs match fan-in: {:?} and {:?}",
            pattern_dff, design_dff
        );
        let data_matches = self.values_match_fan_in(&pattern_dff.data, &design_dff.data);
        let clock_matches = self.control_nets_match_fan_in(&pattern_dff.clock, &design_dff.clock);
        let clear_matches = self.control_nets_match_fan_in(&pattern_dff.clear, &design_dff.clear);
        let reset_matches = self.control_nets_match_fan_in(&pattern_dff.reset, &design_dff.reset);
        let enable_matches =
            self.control_nets_match_fan_in(&pattern_dff.enable, &design_dff.enable);
        let clear_value_matches =
            self.const_match_fan_in(&pattern_dff.clear_value, &design_dff.clear_value);
        let reset_value_matches =
            self.const_match_fan_in(&pattern_dff.reset_value, &design_dff.reset_value);
        let init_value_matches =
            self.const_match_fan_in(&pattern_dff.init_value, &design_dff.init_value);

        let value_data_matches = data_matches
            && clock_matches
            && clear_matches
            && reset_matches
            && enable_matches
            && clear_value_matches
            && reset_value_matches
            && init_value_matches;

        match value_data_matches {
            true => {
                trace!("DFFs match fan-in");
            }
            false => {
                trace!("DFFs do not match fan-in");
            }
        }
        value_data_matches
    }
}

impl<'a, 'p, 'd> Constraint<'d> for ConnectivityConstraint<'a, 'p, 'd> {
    fn d_candidate_is_valid(&self, d_cell: &CellWrapper<'d>) -> bool {
        self.is_cell_connectivity_valid(d_cell.clone())
    }
}
