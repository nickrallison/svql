use crate::constraints::Constraint;
use crate::graph_index::GraphIndex;
use crate::isomorphism::NodeMapping;
use crate::node::{NodeFanin, NodeSource, NodeType};
use crate::profiling::Timer;
use prjunnamed_netlist::{Cell, CellRef};
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
}

impl<'a, 'p, 'd> Constraint<'d> for ConnectivityConstraint<'a, 'p, 'd> {
    fn d_candidate_is_valid(&self, d_node: &CellRef<'d>) -> bool {
        self.is_node_connectivity_valid(*d_node)
    }
}

// enum Cell {
//     Buf(Value),
//     Not(Value),
//     /// `a & b`.
//     ///
//     /// Has short-circuiting behavior for inputs containing `X` — if the other
//     /// bit is `0`, the output is `0` and the `X` doesn't propagate.
//     And(Value, Value),
//     /// `a | b`.
//     ///
//     /// Has short-circuiting behavior for inputs containing `X` — if the other
//     /// bit is `1`, the output is `1` and the `X` doesn't propagate.
//     Or(Value, Value),
//     Xor(Value, Value),
//     /// `a ? b : c`.
//     ///
//     /// Muxes are glitch free — if `a` is `X`, the bit positions that match
//     /// between `b` and `c` still have a defined value. The `X` propagates
//     /// only at the positions where `b` and `c` differ.
//     Mux(Net, Value, Value),
//     /// `a + b + ci` — add with carry.
//     ///
//     /// Output is one bit wider than `a` and `b` — the most significant bit
//     /// is the carry-out.
//     ///
//     /// `X`s in the input propagate only to the more significant bits, and
//     /// do not affect the less significant bits.
//     Adc(Value, Value, Net), // a + b + ci
//     /// `a & b`, single-bit wide, both inputs freely invertible.
//     ///
//     /// A variant of the `And` cell meant for fine logic optimization.
//     Aig(ControlNet, ControlNet),

//     Eq(Value, Value),
//     ULt(Value, Value),
//     SLt(Value, Value),

//     /// `a << (b * c)`. The bottom bits are filled with zeros.
//     ///
//     /// General notes for all shift cells:
//     /// - output is the same width as `a`. If you need wider output,
//     ///   zero-extend or sign-extend your input first, as appropriate.
//     /// - the shift count does not wrap. If you shift by more than
//     ///   `a.len() - 1`, you get the same result as if you made an equivalent
//     ///   sequence of 1-bit shifts (i.e. all zeros, all sign bits, or all `X`,
//     ///   as appropriate).
//     /// - shift cells are one of the few cells which *do not* expect their
//     ///   inputs to be of the same width. In fact, that is the expected case.
//     Shl(Value, Value, u32),
//     /// `a >> (b * c)`. The top bits are filled with zeros.
//     ///
//     /// See also [general notes above][Cell::Shl].
//     UShr(Value, Value, u32),
//     /// `a >> (b * c)`. The top bits are filled with copies of the top bit
//     /// of the input.
//     ///
//     /// `a` must be at least one bit wide (as otherwise there would be no sign
//     /// bit to propagate, and while there wouldn't be anywhere to propagate it
//     /// *to*, it's an edge-case it doesn't make sense to bother handling).
//     ///
//     /// See also [general notes above][Cell::Shl].
//     SShr(Value, Value, u32),
//     /// `a >> (b * c)`. The top bits are filled with `X`.
//     ///
//     /// See also [general notes above][Cell::Shl].
//     XShr(Value, Value, u32),

//     // future possibilities: popcnt, count leading/trailing zeros, powers
//     Mul(Value, Value),
//     UDiv(Value, Value),
//     UMod(Value, Value),
//     SDivTrunc(Value, Value),
//     SDivFloor(Value, Value),
//     SModTrunc(Value, Value),
//     SModFloor(Value, Value),

//     Match(MatchCell),
//     Assign(AssignCell),

//     Dff(FlipFlop),
//     Memory(Memory),
//     IoBuf(IoBuffer),
//     Target(TargetCell),
//     Other(Instance),

//     /// Design input of a given width.
//     ///
//     /// If synthesizing for a specified target, and not in out-of-context mode,
//     /// an input will be replaced with an [`IoBuffer`] and attached to a pin on
//     /// the target device.
//     Input(String, usize),
//     /// Design output. Attaches a name to a given value.
//     ///
//     /// If synthesizing for a specified target, and not in out-of-context mode,
//     /// an output will be replaced with an [`IoBuffer`] and attached to a pin on
//     /// the target device.
//     Output(String, Value),
//     /// Attaches a name to a given value for debugging.
//     ///
//     /// `Name` keeps a given value alive during optimization and makes it easily
//     /// available to be poked at during simulation.
//     ///
//     /// Do note that the [`unname` pass][unname], which runs during
//     /// target-dependent synthesis, replaces all `Name` cells with [`Debug`]
//     /// cells.
//     ///
//     /// [unname]: ../prjunnamed_generic/fn.unname.html
//     /// [`Debug`]: Cell::Debug
//     Name(String, Value),
//     /// Tentatively attaches a name to a given value.
//     ///
//     /// `Debug` gives a name to a particular value, without insisting on keeping
//     /// it alive during optimization. This helps correlate the output of
//     /// synthesis with the corresponding input logic.
//     ///
//     /// If at any point a value is being kept alive only by a `Debug` cell,
//     /// it will be optimized out and the input to the `Debug` cell will
//     /// be replaced with `X`.
//     ///
//     /// See also: [`Name`][Cell::Name].
//     Debug(String, Value),
// }

fn cells_match(pattern_cell: &Cell, design_cell: &Cell) -> bool {
    use Cell::*;
    match (pattern_cell, design_cell) {
        (Buf(pv), Buf(dv)) => todo!(),
        (Not(pv), Not(dv)) => todo!(),
        (And(pa, pb), And(da, db)) => todo!(),
        (Or(pa, pb), Or(da, db)) => todo!(),
        (Xor(pa, pb), Xor(da, db)) => todo!(),
        (Mux(pa, pb, pc), Mux(da, db, dc)) => todo!(),
        (Adc(pa, pb, pci), Adc(da, db, dci)) => todo!(),
        (Aig(pa, pb), Aig(da, db)) => todo!(),
        (Eq(pa, pb), Eq(da, db)) => todo!(),
        (ULt(pa, pb), ULt(da, db)) => todo!(),
        (SLt(pa, pb), SLt(da, db)) => todo!(),
        (Shl(pa, pb, pc), Shl(da, db, dc)) => todo!(),
        (UShr(pa, pb, pc), UShr(da, db, dc)) => todo!(),
        (SShr(pa, pb, pc), SShr(da, db, dc)) => todo!(),
        (XShr(pa, pb, pc), XShr(da, db, dc)) => todo!(),
        (Mul(pa, pb), Mul(da, db)) => todo!(),
        (UDiv(pa, pb), UDiv(da, db)) => todo!(),
        (UMod(pa, pb), UMod(da, db)) => todo!(),
        (SDivTrunc(pa, pb), SDivTrunc(da, db)) => todo!(),
        (SDivFloor(pa, pb), SDivFloor(da, db)) => todo!(),
        (SModTrunc(pa, pb), SModTrunc(da, db)) => todo!(),
        (SModFloor(pa, pb), SModFloor(da, db)) => todo!(),
        (Match(pm), Match(dm)) => todo!(),
        (Assign(pa), Assign(da)) => todo!(),
        (Dff(pd), Dff(dd)) => todo!(),
        (Memory(pm), Memory(dm)) => todo!(),
        // (IoBuf(pi), IoBuf(di)) => todo!(),
        // (Target(pt), Target(dt)) => todo!(),
        // (Other(po), Other(do_)) => todo!(),
        (Input(pn, pw), Input(dn, dw)) => todo!(),
        // (Output(pn, pv), Output(dn, dv)) => todo!(),
        // (Name(pn, pv), Name(dn, dv)) => todo!(),
        // (Debug(pn, pv), Debug(dn, dv)) => todo!(),
        _ => false,
    }
}
