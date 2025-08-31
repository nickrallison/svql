use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::{Cell, CellRef, Design};

use crate::{
    node::{NodeFanin, NodeSource, NodeType, fanin_named, net_to_source},
    profiling::Timer,
};

#[derive(Clone, Debug)]
pub(super) struct GraphIndex<'a> {
    /// Nodes of design in topological order (Name nodes filtered out)
    nodes_topo: Vec<CellRef<'a>>,
    by_type: HashMap<NodeType, Vec<CellRef<'a>>>,
    /// For each driver (key), the list of (sink, sink_pin_idx)
    reverse_node_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>>,

    /// Pre-computed source information for each node: sources[sink][pin_idx] = NodeSource
    node_sources: HashMap<CellRef<'a>, Vec<NodeSource<'a>>>,

    /// fanout membership: driver -> (sink -> set_of_pins)
    fanout_map: HashMap<CellRef<'a>, HashMap<CellRef<'a>, HashSet<usize>>>,

    /// Named fan-in (by port name) for each node.
    fanin_named: HashMap<CellRef<'a>, NodeFanin<'a>>,

    gate_count: usize,
}

impl<'a> GraphIndex<'a> {
    #[contracts::debug_ensures(ret.gate_count() <= design.iter_cells().count())]
    pub(super) fn build(design: &'a Design) -> Self {
        let _t = Timer::new("GraphIndex::build");

        let mut by_type: HashMap<NodeType, Vec<CellRef<'a>>> = HashMap::new();
        let nodes_topo: Vec<CellRef<'a>> = design
            .iter_cells_topo()
            .rev()
            .filter(|cell_ref| {
                let node_type = NodeType::from(cell_ref.get().as_ref());
                !matches!(node_type, NodeType::Name)
            })
            .collect();

        for node_ref in nodes_topo.iter().cloned() {
            let node_type = NodeType::from(node_ref.get().as_ref());
            by_type.entry(node_type).or_default().push(node_ref);
        }

        let mut reverse_node_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>> =
            HashMap::new();
        let mut node_sources: HashMap<CellRef<'a>, Vec<NodeSource<'a>>> = HashMap::new();
        let mut fanout_map: HashMap<CellRef<'a>, HashMap<CellRef<'a>, HashSet<usize>>> =
            HashMap::new();
        let mut fanin_named_map: HashMap<CellRef<'a>, NodeFanin<'a>> = HashMap::new();

        let gate_count = nodes_topo
            .iter()
            .filter(|c| NodeType::from(c.get().as_ref()).is_logic_gate())
            .count();

        // Pre-compute source information for all nodes (as sinks) and named fan-in.
        for node_ref in nodes_topo.iter() {
            // positional sources (kept for existing constraints and fanout_map)
            let mut sources: Vec<NodeSource<'a>> = Vec::new();
            node_ref.visit(|net| {
                sources.push(net_to_source(design, net));
            });
            node_sources.insert(*node_ref, sources);

            // named fan-in
            let cell = node_ref.get();
            fanin_named_map.insert(*node_ref, fanin_named(design, cell.as_ref()));
        }

        // Build reverse lookups and O(1) fanout membership map
        for sink_ref in nodes_topo.iter() {
            let sources = node_sources.get(sink_ref).unwrap();
            for (sink_pin_idx, source) in sources.iter().enumerate() {
                let driver_node = match source {
                    NodeSource::Gate(node_ref, _src_bit) => Some(*node_ref),
                    NodeSource::Io(node_ref, _src_bit) => Some(*node_ref),
                    NodeSource::Const(_trit) => None,
                };

                if let Some(driver) = driver_node {
                    // reverse list (driver -> vec of (sink, pin))
                    reverse_node_lookup
                        .entry(driver)
                        .or_default()
                        .push((*sink_ref, sink_pin_idx));

                    // fanout_map (driver -> map sink -> set_of_pins)
                    let entry = fanout_map.entry(driver).or_default();
                    entry.entry(*sink_ref).or_default().insert(sink_pin_idx);
                }
            }
        }

        GraphIndex {
            nodes_topo,
            by_type,
            reverse_node_lookup,
            node_sources,
            fanout_map,
            fanin_named: fanin_named_map,
            gate_count,
        }
    }

    pub(super) fn gate_count(&self) -> usize {
        self.gate_count
    }

    pub(super) fn get_by_type(&self, node_type: NodeType) -> &[CellRef<'a>] {
        let _t = Timer::new("GraphIndex::get_by_type");
        self.by_type
            .get(&node_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub(super) fn get_nodes_topo(&self) -> &[CellRef<'a>] {
        self.nodes_topo.as_slice()
    }

    pub(super) fn get_fanouts(&self, node: CellRef<'a>) -> &[(CellRef<'a>, usize)] {
        let _t = Timer::new("GraphIndex::get_fanouts");
        let slice = self
            .reverse_node_lookup
            .get(&node)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        slice
    }

    pub(super) fn get_node_sources(&self, node: CellRef<'a>) -> &[NodeSource<'a>] {
        let _t = Timer::new("GraphIndex::get_node_sources");
        self.node_sources
            .get(&node)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub(super) fn get_input_name(&self, node: CellRef<'a>) -> Option<&'a str> {
        let _t = Timer::new("GraphIndex::get_input_name");
        match node.get() {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub(super) fn get_output_name(&self, node: CellRef<'a>) -> Option<&'a str> {
        let _t = Timer::new("GraphIndex::get_output_name");
        match node.get() {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub(super) fn node_summary(&self, node: CellRef<'a>) -> String {
        let _t = Timer::new("GraphIndex::node_summary");
        let node_type = NodeType::from(node.get().as_ref());
        let iname = self.get_input_name(node).unwrap_or("");
        let oname = self.get_output_name(node).unwrap_or("");
        let n = if !iname.is_empty() { iname } else { oname };
        if n.is_empty() {
            format!("#{} {:?}", node.debug_index(), node_type)
        } else {
            format!("#{} {:?}({})", node.debug_index(), node_type, n)
        }
    }

    /// Named fan‑in accessor: returns a named port -> sources map (by bit).
    pub(super) fn get_node_fanin_named(&self, node: CellRef<'a>) -> &NodeFanin<'a> {
        // Constructed for all nodes in build(); unwrap is safe.
        self.fanin_named
            .get(&node)
            .expect("missing NodeFanin for node")
    }

    /// True if `driver` has any fanout edge to `sink` (any input pin).
    pub(super) fn has_fanout_to(&self, driver: CellRef<'a>, sink: CellRef<'a>) -> bool {
        let _t = Timer::new("GraphIndex::has_fanout_to");
        self.fanout_map
            .get(&driver)
            .and_then(|m| m.get(&sink))
            .is_some()
    }

    /// True if `driver` feeds `sink` specifically at `pin_idx`.
    pub(super) fn has_fanout_to_pin(
        &self,
        driver: CellRef<'a>,
        sink: CellRef<'a>,
        pin_idx: usize,
    ) -> bool {
        let _t = Timer::new("GraphIndex::has_fanout_to_pin");
        self.fanout_map
            .get(&driver)
            .and_then(|m| m.get(&sink))
            .is_some_and(|pins| pins.contains(&pin_idx))
    }

    /// The unique driver of a given sink input pin, if any (Gate/Io sources only).
    pub(super) fn driver_of_sink_pin(
        &self,
        sink: CellRef<'a>,
        pin_idx: usize,
    ) -> Option<CellRef<'a>> {
        let _t = Timer::new("GraphIndex::driver_of_sink_pin");
        let src = self.get_node_sources(sink).get(pin_idx)?;
        match src {
            NodeSource::Gate(c, _) | NodeSource::Io(c, _) => Some(*c),
            NodeSource::Const(_) => None,
        }
    }

    /// All drivers of a sink across all pins (Gate/Io only), duplicates removed.
    pub(super) fn drivers_of_sink_all_pins(&self, sink: CellRef<'a>) -> Vec<CellRef<'a>> {
        let _t = Timer::new("GraphIndex::drivers_of_sink_all_pins");
        let mut out: Vec<CellRef<'a>> = self
            .get_node_sources(sink)
            .iter()
            .filter_map(|src| match src {
                NodeSource::Gate(c, _) | NodeSource::Io(c, _) => Some(*c),
                NodeSource::Const(_) => None,
            })
            .collect();

        out.sort_by_key(|c| c.debug_index());
        out.dedup();
        out
    }
}
