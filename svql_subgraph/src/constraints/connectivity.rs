use crate::constraints::Constraint;
use crate::graph_index::GraphIndex;
use crate::isomorphism::NodeMapping;
use crate::node::{NodeFanin, NodeSource, NodeType};
use crate::profiling::Timer;
use prjunnamed_netlist::{Cell, CellRef, Value, ValueRepr};
use svql_common::Config;
use tracing::{debug, trace};

pub(crate) struct ConnectivityConstraint<'a, 'p, 'd> {
    p_node: CellRef<'p>,
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    config: &'a Config,
    mapping: NodeMapping<'p, 'd>,
}

impl<'a, 'p, 'd> ConnectivityConstraint<'a, 'p, 'd> {
    pub(crate) fn new(
        p_node: CellRef<'p>,
        pattern_index: &'a GraphIndex<'p>,
        design_index: &'a GraphIndex<'d>,
        config: &'a Config,
        mapping: NodeMapping<'p, 'd>,
    ) -> Self {
        ConnectivityConstraint {
            p_node,
            pattern_index,
            design_index,
            config,
            mapping,
        }
    }

    fn is_node_connectivity_valid(&self, d_node: CellRef<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::is_node_connectivity_valid");
        trace!(
            "Validating connectivity for design node {:?} against pattern node {:?}",
            d_node, self.p_node
        );

        let valid_fanin = self.validate_fanin_connections(d_node);
        let valid_fanout = self.validate_fanout_connections(d_node);

        let result = valid_fanin && valid_fanout;
        if !result {
            debug!(
                "Connectivity validation failed for design node {:?}: fanin={}, fanout={}",
                d_node, valid_fanin, valid_fanout
            );
        }
        result
    }

    fn validate_fanout_connections(&self, d_node: CellRef<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::validate_fanout_connections");
        let p_fanouts = self.pattern_index.get_fanouts(self.p_node);

        trace!(
            "Validating fanout connections for design node {:?}, pattern has {} fanouts",
            d_node,
            p_fanouts.len()
        );

        // Only need to validate edges to already-mapped sinks.
        let result = p_fanouts
            .iter()
            .filter_map(|(p_sink_node, pin_idx)| {
                self.mapping
                    .get_design_node(*p_sink_node)
                    .map(|d_sink_node| (d_sink_node, *pin_idx))
            })
            .all(|(d_sink_node, pin_idx)| self.fanout_edge_ok(d_node, d_sink_node, pin_idx));

        if !result {
            debug!("Fanout validation failed for design node {:?}", d_node);
        }
        result
    }

    fn fanout_edge_ok(
        &self,
        d_driver: prjunnamed_netlist::CellRef<'d>,
        d_sink_node: prjunnamed_netlist::CellRef<'d>,
        pin_idx: usize,
    ) -> bool {
        let _t = Timer::new("ConnectivityConstraint::fanout_edge_ok");
        let d_sink_node_type = NodeType::from(d_sink_node.get().as_ref());
        let sink_commutative = d_sink_node_type.has_commutative_inputs();

        let result = if sink_commutative {
            self.design_index.has_fanout_to(d_driver, d_sink_node)
        } else {
            self.design_index
                .has_fanout_to_pin(d_driver, d_sink_node, pin_idx)
        };

        if !result {
            trace!(
                "Fanout edge check failed: driver {:?} -> sink {:?} pin {}, commutative: {}",
                d_driver, d_sink_node, pin_idx, sink_commutative
            );
        }
        result
    }

    // NEW: Named-port fan-in validation
    fn validate_fanin_connections(&self, d_node: CellRef<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::validate_fanin_connections");

        let p_fanin: &NodeFanin<'p> = self.pattern_index.get_node_fanin_named(self.p_node);
        let d_fanin: &NodeFanin<'d> = self.design_index.get_node_fanin_named(d_node);

        trace!(
            "Validating fanin for design node {:?}, pattern has {} fanin ports",
            d_node,
            p_fanin.map.len()
        );

        // All named ports in the pattern must exist in the candidate, with the same bit widths.
        for (p_name, p_sources) in p_fanin.map.iter() {
            let Some(d_sources) = d_fanin.map.get(p_name) else {
                debug!(
                    "Fanin validation failed: design node {:?} missing port {}",
                    d_node, p_name
                );
                return false;
            };
            if d_sources.len() != p_sources.len() && self.config.match_length {
                debug!(
                    "Fanin validation failed: design node {:?} port {} width mismatch on exact length match (\npattern: {}\ndesign: {})",
                    d_node,
                    p_name,
                    p_sources.len(),
                    d_sources.len()
                );
                return false;
            }

            if d_sources.len() < p_sources.len() && !self.config.match_length {
                debug!(
                    "Fanin validation failed: design node {:?} port {} width mismatch on superset length match (\npattern: {}\ndesign: {})",
                    d_node,
                    p_name,
                    p_sources.len(),
                    d_sources.len()
                );
                return false;
            }

            // Bit-by-bit compatibility using existing mapping (unmapped pattern sources are unconstrained)
            for (i, (p_src, d_src)) in p_sources.iter().zip(d_sources.iter()).enumerate() {
                if !self.sources_compatible(p_src, d_src) {
                    debug!(
                        "Fanin validation failed: design node {:?} port {} bit {} source incompatible",
                        d_node, p_name, i
                    );
                    return false;
                }
            }
        }

        true
    }

    fn sources_compatible(&self, p_src: &NodeSource<'p>, d_src: &NodeSource<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::sources_compatible");
        match p_src {
            NodeSource::Const(pt) => matches!(d_src, NodeSource::Const(dt) if dt == pt),

            // Gate/Io sources must map to the mapped design node (if mapping exists yet).
            NodeSource::Gate(p_node, p_bit) | NodeSource::Io(p_node, p_bit) => {
                if let Some(d_expected) = self.mapping.get_design_node(*p_node) {
                    match d_src {
                        NodeSource::Gate(d_node, d_bit) | NodeSource::Io(d_node, d_bit) => {
                            *d_node == d_expected && *d_bit == *p_bit
                        }
                        NodeSource::Const(_) => false,
                    }
                } else {
                    // If the pattern source isn't mapped yet, we don't constrain it here.
                    true
                }
            }
        }
    }

    // ####################################
    fn cells_match_fan_in(&self, pattern_cell: &Cell, design_cell: &Cell) -> bool {
        use Cell::*;
        match (pattern_cell, design_cell) {
            (Buf(p_value), Buf(d_value)) => todo!(),
            (Not(p_value), Not(d_value)) => todo!(),
            (And(p_a_value, p_b_value), And(d_a_value, d_b_value)) => todo!(),
            (Or(p_a_value, p_b_value), Or(d_a_value, d_b_value)) => todo!(),
            (Xor(p_a_value, p_b_value), Xor(d_a_value, d_b_value)) => todo!(),
            (Mux(p_a_value, p_b_value, p_c_value), Mux(d_a_value, d_b_value, d_c_value)) => todo!(),
            (Adc(p_a_value, p_b_value, p_ci_net), Adc(d_a_value, d_b_value, d_ci_net)) => todo!(),
            (Aig(pa_control_net, pb_control_net), Aig(da_control_net, db_control_net)) => todo!(),
            (Eq(pa_value, pb_value), Eq(da_value, db_value)) => todo!(),
            (ULt(pa_value, pb_value), ULt(da_value, db_value)) => todo!(),
            (SLt(pa_value, pb_value), SLt(da_value, db_value)) => todo!(),
            (Shl(pa_value, pb_value, pc_u32), Shl(da_value, db_value, dc_u32)) => todo!(),
            (UShr(pa_value, pb_value, pc_u32), UShr(da_value, db_value, dc_u32)) => todo!(),
            (SShr(pa_value, pb_value, pc_u32), SShr(da_value, db_value, dc_u32)) => todo!(),
            (XShr(pa_value, pb_value, pc_u32), XShr(da_value, db_value, dc_u32)) => todo!(),
            (Mul(pa_value, pb_value), Mul(da_value, db_value)) => todo!(),
            (UDiv(pa_value, pb_value), UDiv(da_value, db_value)) => todo!(),
            (UMod(pa_value, pb_value), UMod(da_value, db_value)) => todo!(),
            (SDivTrunc(pa_value, pb_value), SDivTrunc(da_value, db_value)) => todo!(),
            (SDivFloor(pa_value, pb_value), SDivFloor(da_value, db_value)) => todo!(),
            (SModTrunc(pa_value, pb_value), SModTrunc(da_value, db_value)) => todo!(),
            (SModFloor(pa_value, pb_value), SModFloor(da_value, db_value)) => todo!(),
            (Match(p_match_cell), Match(d_match_cell)) => todo!(),
            (Assign(p_assign_cell), Assign(d_assign_cell)) => todo!(),
            (Dff(p_dff_cell), Dff(d_dff_cell)) => todo!(),
            (Memory(p_memory_cell), Memory(d_memory_cell)) => todo!(),
            // (IoBuf(pi), IoBuf(di)) => todo!(),
            // (Target(pt), Target(dt)) => todo!(),
            // (Other(po), Other(do_)) => todo!(),
            (Input(p_name, p_width), Input(d_name, d_width)) => todo!(),
            // (Output(pn, pv), Output(dn, dv)) => todo!(),
            // (Name(pn, pv), Name(dn, dv)) => todo!(),
            // (Debug(pn, pv), Debug(dn, dv)) => todo!(),
            _ => false,
        }
    }

    fn values_match_fan_in(&self, pattern_value: &Value, design_value: &Value) -> bool {
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
            _ => todo!("Should single pattern match against many design"),
        }
    }

    fn nets_match_fan_in(
        &self,
        pattern_net: &prjunnamed_netlist::Net,
        design_net: &prjunnamed_netlist::Net,
    ) -> bool {
        todo!("Look up net values in designs & make sure they correspond to mapped cells")
    }
}

impl<'a, 'p, 'd> Constraint<'d> for ConnectivityConstraint<'a, 'p, 'd> {
    fn d_candidate_is_valid(&self, d_node: &CellRef<'d>) -> bool {
        self.is_node_connectivity_valid(*d_node)
    }
}
