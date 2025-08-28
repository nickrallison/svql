use crate::constraints::Constraint;
use crate::graph_index::GraphIndex;
use crate::isomorphism::NodeMapping;
use crate::node::{NodeSource, NodeType};
use prjunnamed_netlist::CellRef;
use svql_common::Config;

pub(crate) struct ConnectivityConstraint<'a, 'p, 'd> {
    p_node: CellRef<'p>,
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    _config: &'a Config,
    mapping: &'a NodeMapping<'p, 'd>,
}

impl<'a, 'p, 'd> ConnectivityConstraint<'a, 'p, 'd> {
    pub(crate) fn new(
        p_node: CellRef<'p>,
        pattern_index: &'a GraphIndex<'p>,
        design_index: &'a GraphIndex<'d>,
        config: &'a Config,
        mapping: &'a NodeMapping<'p, 'd>,
    ) -> Self {
        ConnectivityConstraint {
            p_node,
            pattern_index,
            design_index,
            _config: config,
            mapping,
        }
    }

    fn is_node_connectivity_valid(&self, d_node: CellRef<'d>) -> bool {
        let valid_fanin = self.validate_fanin_connections(d_node);
        let valid_fanout = self.validate_fanout_connections(d_node);

        valid_fanin && valid_fanout
    }

    fn validate_fanout_connections(&self, d_node: CellRef<'d>) -> bool {
        let p_fanouts = self.pattern_index.get_fanouts(self.p_node);

        // Only need to validate edges to already-mapped sinks.
        p_fanouts
            .iter()
            .filter_map(|(p_sink_node, pin_idx)| {
                self.mapping
                    .get_design_node(*p_sink_node)
                    .map(|d_sink_node| (d_sink_node, *pin_idx))
            })
            .all(|(d_sink_node, pin_idx)| self.fanout_edge_ok(d_node, d_sink_node, pin_idx))
    }

    fn fanout_edge_ok(
        &self,
        d_driver: prjunnamed_netlist::CellRef<'d>,
        d_sink_node: prjunnamed_netlist::CellRef<'d>,
        pin_idx: usize,
    ) -> bool {
        let d_sink_node_type = NodeType::from(d_sink_node.get().as_ref());
        let sink_commutative = d_sink_node_type.has_commutative_inputs();

        let ok = if sink_commutative {
            self.design_index.has_fanout_to(d_driver, d_sink_node)
        } else {
            self.design_index
                .has_fanout_to_pin(d_driver, d_sink_node, pin_idx)
        };

        tracing::event!(
            tracing::Level::TRACE,
            "is_node_connectivity_valid: check mapped sink D#{} @pin={} (commutative={}) -> {}",
            d_sink_node.debug_index(),
            pin_idx,
            sink_commutative,
            ok
        );

        ok
    }

    fn validate_fanin_connections(&self, d_node: CellRef<'d>) -> bool {
        let p_sources = self.pattern_index.get_node_sources(self.p_node);
        let d_sources = self.design_index.get_node_sources(d_node);

        p_sources
            .iter()
            .enumerate()
            .all(|(pin_idx, p_src)| match d_sources.get(pin_idx) {
                Some(d_src) => self.pin_sources_compatible(p_src, d_src),
                None => false,
            })
    }

    fn pin_sources_compatible(&self, p_src: &NodeSource<'p>, d_src: &NodeSource<'d>) -> bool {
        match p_src {
            NodeSource::Const(_) => {
                matches!(d_src, NodeSource::Const(dt) if matches!(p_src, NodeSource::Const(pt) if dt == pt))
            }
            NodeSource::Io(p_src_node, p_bit) => {
                self.source_matches_mapped_io(*p_src_node, *p_bit, d_src)
            }
            NodeSource::Gate(p_src_node, p_bit) => {
                self.source_matches_mapped_gate(*p_src_node, *p_bit, d_src)
            }
        }
    }

    fn source_matches_mapped_io(
        &self,
        p_src_node: prjunnamed_netlist::CellRef<'p>,
        p_bit: usize,
        d_src: &NodeSource<'d>,
    ) -> bool {
        let Some(d_src_node) = self.mapping.get_design_node(p_src_node) else {
            // Unmapped pattern source; unconstrained at this stage.
            return true;
        };

        match d_src {
            NodeSource::Io(d_node, d_bit) => *d_node == d_src_node && *d_bit == p_bit,
            NodeSource::Gate(d_node, d_bit) => *d_node == d_src_node && *d_bit == p_bit,
            NodeSource::Const(_) => false,
        }
    }

    fn source_matches_mapped_gate(
        &self,
        p_src_node: prjunnamed_netlist::CellRef<'p>,
        p_bit: usize,
        d_src: &NodeSource<'d>,
    ) -> bool {
        let Some(d_src_node) = self.mapping.get_design_node(p_src_node) else {
            // Unmapped pattern source; unconstrained at this stage.
            return true;
        };

        matches!(d_src, NodeSource::Gate(d_node, d_bit) if *d_node == d_src_node && *d_bit == p_bit)
    }
}

impl<'a, 'p, 'd> Constraint<'d> for ConnectivityConstraint<'a, 'p, 'd> {
    fn d_candidate_is_valid(&self, d_node: &CellRef<'d>) -> bool {
        self.is_node_connectivity_valid(*d_node)
    }
}
