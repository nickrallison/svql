use crate::SubgraphMatcherCore;
use crate::cell::CellWrapper;
use crate::mapping::Mapping;
use prjunnamed_netlist::{Cell, CellRef, FlipFlop, Trit, Value, ValueRepr};
use tracing::trace;

impl<'p, 'd, 'a> SubgraphMatcherCore<'p, 'd, 'a> {
    pub(crate) fn validate_fan_in_connections(
        &self,
        p_cell: CellWrapper<'p>,
        d_cell: CellWrapper<'d>,
        mapping: &Mapping<'p, 'd>,
    ) -> bool {
        self.cells_match_fan_in(p_cell.get(), d_cell.get(), mapping)
    }

    // ####################################
    fn cells_match_fan_in(
        &self,
        pattern_cell: &Cell,
        design_cell: &Cell,
        mapping: &Mapping<'p, 'd>,
    ) -> bool {
        use Cell::*;
        match (pattern_cell, design_cell) {
            (Buf(p_value), Buf(d_value)) => self.values_match_fan_in(p_value, d_value, mapping),
            (Not(p_value), Not(d_value)) => self.values_match_fan_in(p_value, d_value, mapping),
            (And(p_a_value, p_b_value), And(d_a_value, d_b_value)) => {
                let a_matches = self.values_match_fan_in(p_a_value, d_a_value, mapping);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value, mapping);
                let result_normal = a_matches && b_matches;

                // Commutative case
                let a_swapped = self.values_match_fan_in(p_a_value, d_b_value, mapping);
                let b_swapped = self.values_match_fan_in(p_b_value, d_a_value, mapping);
                let result_swapped = a_swapped && b_swapped;

                result_normal || result_swapped
            }
            (Or(p_a_value, p_b_value), Or(d_a_value, d_b_value)) => {
                let a_matches = self.values_match_fan_in(p_a_value, d_a_value, mapping);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value, mapping);
                let result_normal = a_matches && b_matches;

                // Commutative case
                let a_swapped = self.values_match_fan_in(p_a_value, d_b_value, mapping);
                let b_swapped = self.values_match_fan_in(p_b_value, d_a_value, mapping);
                let result_swapped = a_swapped && b_swapped;

                result_normal || result_swapped
            }
            (Xor(p_a_value, p_b_value), Xor(d_a_value, d_b_value)) => {
                let a_matches = self.values_match_fan_in(p_a_value, d_a_value, mapping);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value, mapping);
                let result_normal = a_matches && b_matches;

                // Commutative case
                let a_swapped = self.values_match_fan_in(p_a_value, d_b_value, mapping);
                let b_swapped = self.values_match_fan_in(p_b_value, d_a_value, mapping);
                let result_swapped = a_swapped && b_swapped;

                result_normal || result_swapped
            }
            (Mux(p_a_value, p_b_value, p_c_value), Mux(d_a_value, d_b_value, d_c_value)) => {
                let a_matches = self.nets_match_fan_in(p_a_value, d_a_value, mapping);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value, mapping);
                let c_matches = self.values_match_fan_in(p_c_value, d_c_value, mapping);
                a_matches && b_matches && c_matches
            }
            (Adc(p_a_value, p_b_value, p_ci_net), Adc(d_a_value, d_b_value, d_ci_net)) => {
                let a_matches = self.values_match_fan_in(p_a_value, d_a_value, mapping);
                let b_matches = self.values_match_fan_in(p_b_value, d_b_value, mapping);
                let ci_matches = self.nets_match_fan_in(p_ci_net, d_ci_net, mapping);
                a_matches && b_matches && ci_matches
            }
            (Aig(pa_control_net, pb_control_net), Aig(da_control_net, db_control_net)) => {
                let a_matches =
                    self.control_nets_match_fan_in(pa_control_net, da_control_net, mapping);
                let b_matches =
                    self.control_nets_match_fan_in(pb_control_net, db_control_net, mapping);
                a_matches && b_matches
            }
            (Eq(pa_value, pb_value), Eq(da_value, db_value)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value, mapping);
                let b_matches = self.values_match_fan_in(pb_value, db_value, mapping);
                a_matches && b_matches
            }
            (ULt(pa_value, pb_value), ULt(da_value, db_value)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value, mapping);
                let b_matches = self.values_match_fan_in(pb_value, db_value, mapping);
                a_matches && b_matches
            }
            (SLt(pa_value, pb_value), SLt(da_value, db_value)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value, mapping);
                let b_matches = self.values_match_fan_in(pb_value, db_value, mapping);
                a_matches && b_matches
            }
            (Shl(pa_value, pb_value, pc_u32), Shl(da_value, db_value, dc_u32)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value, mapping);
                let b_matches = self.values_match_fan_in(pb_value, db_value, mapping);
                let c_matches = pc_u32 == dc_u32;
                a_matches && b_matches && c_matches
            }
            (UShr(pa_value, pb_value, pc_u32), UShr(da_value, db_value, dc_u32)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value, mapping);
                let b_matches = self.values_match_fan_in(pb_value, db_value, mapping);
                let c_matches = pc_u32 == dc_u32;
                a_matches && b_matches && c_matches
            }
            (SShr(pa_value, pb_value, pc_u32), SShr(da_value, db_value, dc_u32)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value, mapping);
                let b_matches = self.values_match_fan_in(pb_value, db_value, mapping);
                let c_matches = pc_u32 == dc_u32;
                a_matches && b_matches && c_matches
            }
            (XShr(pa_value, pb_value, pc_u32), XShr(da_value, db_value, dc_u32)) => {
                let a_matches = self.values_match_fan_in(pa_value, da_value, mapping);
                let b_matches = self.values_match_fan_in(pb_value, db_value, mapping);
                let c_matches = pc_u32 == dc_u32;
                a_matches && b_matches && c_matches
            }
            (Mul(pa_value, pb_value), Mul(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value, mapping)
                    && self.values_match_fan_in(pb_value, db_value, mapping)
            }
            (UDiv(pa_value, pb_value), UDiv(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value, mapping)
                    && self.values_match_fan_in(pb_value, db_value, mapping)
            }
            (UMod(pa_value, pb_value), UMod(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value, mapping)
                    && self.values_match_fan_in(pb_value, db_value, mapping)
            }
            (SDivTrunc(pa_value, pb_value), SDivTrunc(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value, mapping)
                    && self.values_match_fan_in(pb_value, db_value, mapping)
            }
            (SDivFloor(pa_value, pb_value), SDivFloor(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value, mapping)
                    && self.values_match_fan_in(pb_value, db_value, mapping)
            }
            (SModTrunc(pa_value, pb_value), SModTrunc(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value, mapping)
                    && self.values_match_fan_in(pb_value, db_value, mapping)
            }
            (SModFloor(pa_value, pb_value), SModFloor(da_value, db_value)) => {
                self.values_match_fan_in(pa_value, da_value, mapping)
                    && self.values_match_fan_in(pb_value, db_value, mapping)
            }
            (Match(_p_match_cell), Match(_d_match_cell)) => {
                todo!("Make Function to match match cells")
            }
            (Assign(_p_assign_cell), Assign(_d_assign_cell)) => {
                todo!("Make Function to match assign cells")
            }
            (Dff(p_dff_cell), Dff(d_dff_cell)) => {
                self.dffs_match_fan_in(p_dff_cell, d_dff_cell, mapping)
            }
            (Memory(_p_memory_cell), Memory(_d_memory_cell)) => {
                todo!("Make Function to match memory cells")
            }
            // (IoBuf(pi), IoBuf(di)) => todo!(),
            // (Target(pt), Target(dt)) => todo!(),
            // (Other(po), Other(do_)) => todo!(),
            (Input(_p_name, _p_width), Input(_d_name, _d_width)) => {
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

    fn values_match_fan_in(
        &self,
        pattern_value: &Value,
        design_value: &Value,
        mapping: &Mapping<'p, 'd>,
    ) -> bool {
        trace!(
            "Checking if values match fan-in: {:?} and {:?}",
            pattern_value, design_value
        );
        let pattern_value_repr: &ValueRepr = &pattern_value.0;
        let design_value_repr: &ValueRepr = &design_value.0;
        match (pattern_value_repr, design_value_repr) {
            (ValueRepr::None, ValueRepr::None) => true,
            (ValueRepr::Some(p_net), ValueRepr::Some(d_net)) => {
                self.nets_match_fan_in(p_net, d_net, mapping)
            }
            (ValueRepr::Many(p_nets), ValueRepr::Many(d_nets)) => match self.config.match_length {
                svql_common::MatchLength::First => {
                    let first_p_net = p_nets.first().unwrap();
                    let first_d_net = d_nets.first().unwrap();
                    return self.nets_match_fan_in(first_p_net, first_d_net, mapping);
                }
                svql_common::MatchLength::NeedleSubsetHaystack => {
                    for p_net in p_nets {
                        let mut found_match = false;
                        for d_net in d_nets {
                            if self.nets_match_fan_in(p_net, d_net, mapping) {
                                found_match = true;
                                break;
                            }
                        }
                        if !found_match {
                            return false;
                        }
                    }
                    return true;
                }
                svql_common::MatchLength::Exact => {
                    if p_nets.len() != d_nets.len() {
                        return false;
                    }
                    for (p_net, d_net) in p_nets.iter().zip(d_nets.iter()) {
                        if !self.nets_match_fan_in(p_net, d_net, mapping) {
                            return false;
                        }
                    }
                    return true;
                }
            },
            (ValueRepr::Some(p_net), ValueRepr::Many(d_nets)) => match self.config.match_length {
                svql_common::MatchLength::First => {
                    let first_d_net = d_nets.first().unwrap();
                    return self.nets_match_fan_in(p_net, first_d_net, mapping);
                }
                svql_common::MatchLength::NeedleSubsetHaystack => {
                    for d_net in d_nets {
                        if self.nets_match_fan_in(p_net, d_net, mapping) {
                            return true;
                        }
                    }
                    return false;
                }
                svql_common::MatchLength::Exact => {
                    if d_nets.len() != 1 {
                        return false;
                    }
                    let first_d_net = d_nets.first().unwrap();
                    return self.nets_match_fan_in(p_net, first_d_net, mapping);
                }
            },
            _ => false,
        }
    }

    fn nets_match_fan_in(
        &self,
        pattern_net: &prjunnamed_netlist::Net,
        design_net: &prjunnamed_netlist::Net,
        mapping: &Mapping<'p, 'd>,
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
            mapping.get_design_cell(fan_in_pattern_cell_ref.into());

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
        mapping: &Mapping<'p, 'd>,
    ) -> bool {
        trace!(
            "Checking if control nets match fan-in: {:?} and {:?}",
            pattern_c_net, design_c_net
        );
        match (pattern_c_net, design_c_net) {
            (
                prjunnamed_netlist::ControlNet::Pos(p_pos_net),
                prjunnamed_netlist::ControlNet::Pos(d_pos_net),
            ) => self.nets_match_fan_in(p_pos_net, d_pos_net, mapping),
            (
                prjunnamed_netlist::ControlNet::Neg(p_neg_net),
                prjunnamed_netlist::ControlNet::Neg(d_neg_net),
            ) => self.nets_match_fan_in(p_neg_net, d_neg_net, mapping),
            _ => false,
        }
    }

    fn const_match_fan_in(
        &self,
        pattern_const: &prjunnamed_netlist::Const,
        design_const: &prjunnamed_netlist::Const,
        _mapping: &Mapping<'p, 'd>,
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

    fn dffs_match_fan_in(
        &self,
        pattern_dff: &FlipFlop,
        design_dff: &FlipFlop,
        mapping: &Mapping,
    ) -> bool {
        trace!(
            "Checking if DFFs match fan-in: {:?} and {:?}",
            pattern_dff, design_dff
        );
        let data_matches = self.values_match_fan_in(&pattern_dff.data, &design_dff.data, mapping);
        let clock_matches =
            self.control_nets_match_fan_in(&pattern_dff.clock, &design_dff.clock, mapping);
        let clear_matches =
            self.control_nets_match_fan_in(&pattern_dff.clear, &design_dff.clear, mapping);
        let reset_matches =
            self.control_nets_match_fan_in(&pattern_dff.reset, &design_dff.reset, mapping);
        let enable_matches =
            self.control_nets_match_fan_in(&pattern_dff.enable, &design_dff.enable, mapping);
        let clear_value_matches =
            self.const_match_fan_in(&pattern_dff.clear_value, &design_dff.clear_value, mapping);
        let reset_value_matches =
            self.const_match_fan_in(&pattern_dff.reset_value, &design_dff.reset_value, mapping);
        let init_value_matches =
            self.const_match_fan_in(&pattern_dff.init_value, &design_dff.init_value, mapping);

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
