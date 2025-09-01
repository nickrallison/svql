use std::collections::HashSet;
use tracing::trace;

use crate::Timer;
use crate::constraints::node::NodeConstraint;
use crate::isomorphism::NodeMapping;
use crate::node::NodeType;
use crate::{Constraint, GraphIndex};
use prjunnamed_netlist::CellRef;

pub(crate) struct DesignSinkConstraint<'d> {
    node_constraints: NodeConstraint<'d>,
}

impl<'d> DesignSinkConstraint<'d> {
    pub(crate) fn new<'p>(
        pattern_current: CellRef<'p>,
        pattern_index: &GraphIndex<'p>,
        design_index: &GraphIndex<'d>,
        mapping: &NodeMapping<'p, 'd>,
    ) -> Self {
        let _t = Timer::new("DesignSinkConstraint::new");
        // For each mapped fanout sink, gather its possible driver(s), then intersect across sinks.
        let mapped_sinks: Vec<(CellRef<'p>, usize, CellRef<'d>)> = pattern_index
            .get_fanouts(pattern_current)
            .iter()
            .filter_map(|(p_sink_node, pin_idx)| {
                mapping
                    .get_design_node(*p_sink_node)
                    .map(|d_sink_node| (*p_sink_node, *pin_idx, d_sink_node))
            })
            .collect();

        trace!(
            "DesignSinkConstraint for pattern node {:?} found {} mapped sinks",
            pattern_current,
            mapped_sinks.len()
        );

        if mapped_sinks.is_empty() {
            trace!("No mapped sinks, returning unconstrained");
            return DesignSinkConstraint {
                node_constraints: NodeConstraint::new(None),
            };
        }

        let sets = mapped_sinks
            .iter()
            .map(|(_p_sink, pin_idx, d_sink)| {
                let sink_type = NodeType::from(d_sink.get().as_ref());

                if sink_type.has_commutative_inputs() {
                    // Any driver to any pin
                    design_index.drivers_of_sink_all_pins(*d_sink)
                } else {
                    // Specific pin must match
                    design_index
                        .driver_of_sink_pin(*d_sink, *pin_idx)
                        .into_iter()
                        .collect()
                }
            })
            .filter(|v| !v.is_empty())
            .map(|v| v.into_iter().collect::<HashSet<CellRef<'d>>>())
            .map(|s| NodeConstraint::new(Some(s)));

        DesignSinkConstraint {
            node_constraints: NodeConstraint::intersect_many(sets),
        }
    }
    pub(crate) fn get_candidates(&self) -> &NodeConstraint<'d> {
        &self.node_constraints
    }
    pub(crate) fn get_candidates_owned(self) -> NodeConstraint<'d> {
        self.node_constraints
    }
}

impl<'d> Constraint<'d> for DesignSinkConstraint<'d> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool {
        let _t = Timer::new("DesignSinkConstraint::d_candidate_is_valid");
        self.node_constraints.d_candidate_is_valid(node)
    }
}
