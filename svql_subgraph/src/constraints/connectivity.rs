use crate::constraints::Constraint;
use crate::graph_index::GraphIndex;
use crate::isomorphism::NodeMapping;
use crate::node::{NodeFanin, NodeSource, NodeType};
use crate::profiling::Timer;
use prjunnamed_netlist::CellRef;
use svql_common::Config;
use tracing::{debug, trace};

pub(crate) struct ConnectivityConstraint<'a, 'p, 'd> {
    p_node: CellRef<'p>,
    pattern_index: &'a GraphIndex<'p>,
    design_index: &'a GraphIndex<'d>,
    _config: &'a Config,
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
            _config: config,
            mapping,
        }
    }

    fn is_node_connectivity_valid(&self, d_node: CellRef<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::is_node_connectivity_valid");
        trace!("Validating connectivity for design node {:?} against pattern node {:?}", 
               d_node, self.p_node);
        
        let valid_fanin = self.validate_fanin_connections(d_node);
        let valid_fanout = self.validate_fanout_connections(d_node);

        let result = valid_fanin && valid_fanout;
        if !result {
            debug!("Connectivity validation failed for design node {:?}: fanin={}, fanout={}", 
                   d_node, valid_fanin, valid_fanout);
        }
        result
    }

    fn validate_fanout_connections(&self, d_node: CellRef<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::validate_fanout_connections");
        let p_fanouts = self.pattern_index.get_fanouts(self.p_node);
        
        trace!("Validating fanout connections for design node {:?}, pattern has {} fanouts", 
               d_node, p_fanouts.len());

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
            trace!("Fanout edge check failed: driver {:?} -> sink {:?} pin {}, commutative: {}", 
                   d_driver, d_sink_node, pin_idx, sink_commutative);
        }
        result
    }

    // NEW: Named-port fan-in validation
    fn validate_fanin_connections(&self, d_node: CellRef<'d>) -> bool {
        let _t = Timer::new("ConnectivityConstraint::validate_fanin_connections");

        let p_fanin: &NodeFanin<'p> = self.pattern_index.get_node_fanin_named(self.p_node);
        let d_fanin: &NodeFanin<'d> = self.design_index.get_node_fanin_named(d_node);

        trace!("Validating fanin for design node {:?}, pattern has {} fanin ports", 
               d_node, p_fanin.map.len());

        // All named ports in the pattern must exist in the candidate, with the same bit widths.
        for (p_name, p_sources) in p_fanin.map.iter() {
            let Some(d_sources) = d_fanin.map.get(p_name) else {
                debug!("Fanin validation failed: design node {:?} missing port {}", d_node, p_name);
                return false;
            };
            if d_sources.len() != p_sources.len() {
                debug!("Fanin validation failed: design node {:?} port {} width mismatch (pattern: {}, design: {})", 
                       d_node, p_name, p_sources.len(), d_sources.len());
                return false;
            }

            // Bit-by-bit compatibility using existing mapping (unmapped pattern sources are unconstrained)
            for (i, (p_src, d_src)) in p_sources.iter().zip(d_sources.iter()).enumerate() {
                if !self.sources_compatible(p_src, d_src) {
                    debug!("Fanin validation failed: design node {:?} port {} bit {} source incompatible", 
                           d_node, p_name, i);
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
