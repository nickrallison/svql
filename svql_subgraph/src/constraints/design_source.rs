use std::collections::HashSet;
use tracing::trace;

use crate::constraints::node::NodeConstraint;
use crate::isomorphism::NodeMapping;
use crate::node::{NodeSource, NodeType};
use crate::profiling::Timer;
use crate::{Constraint, GraphIndex};
use prjunnamed_netlist::CellRef;

pub(crate) struct DesignSourceConstraint<'d> {
    node_constraints: NodeConstraint<'d>,
}

impl<'d> DesignSourceConstraint<'d> {
    pub(crate) fn new<'p>(
        pattern_current: CellRef<'p>,
        pattern_index: &GraphIndex<'p>,
        design_index: &GraphIndex<'d>,
        mapping: &NodeMapping<'p, 'd>,
    ) -> Self {
        let _t = Timer::new("DesignSourceConstraint::new");
        let current_type = NodeType::from(pattern_current.get().as_ref());
        let commutative = current_type.has_commutative_inputs();

        let mapped_sources: Vec<(usize, NodeSource<'p>)> = pattern_index
            .get_node_sources(pattern_current)
            .iter()
            .cloned()
            .enumerate()
            .collect();

        trace!("DesignSourceConstraint for pattern node {:?} of type {:?}, commutative: {}, {} sources", 
               pattern_current, current_type, commutative, mapped_sources.len());

        let sets = mapped_sources
            .into_iter()
            .filter_map(|(pin_idx, p_src)| match p_src {
                NodeSource::Gate(p_src_node, _pbit) | NodeSource::Io(p_src_node, _pbit) => mapping
                    .get_design_node(p_src_node)
                    .map(|d_src_node| (pin_idx, d_src_node)),
                NodeSource::Const(_) => None, // leave const handling to full connectivity validation
            })
            .map(|(pin_idx, d_src_node)| {
                // For the mapped source driver, get all its fanouts in the design.
                // If commutative, any pin is acceptable; otherwise, the exact pin must match.
                let fanouts = design_index.get_fanouts(d_src_node);
                let sinks = fanouts
                    .iter()
                    .filter(move |(_, sink_pin)| commutative || *sink_pin == pin_idx)
                    .map(|(sink, _)| *sink)
                    .collect::<Vec<_>>();
                sinks
            })
            .filter(|v| !v.is_empty())
            .map(|v| v.into_iter().collect::<HashSet<CellRef<'d>>>())
            .map(|s| NodeConstraint::new(Some(s)));

        DesignSourceConstraint {
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

impl<'d> Constraint<'d> for DesignSourceConstraint<'d> {
    fn d_candidate_is_valid(&self, node: &CellRef<'d>) -> bool {
        let _t = Timer::new("DesignSourceConstraint::d_candidate_is_valid");
        self.node_constraints.d_candidate_is_valid(node)
    }
}
