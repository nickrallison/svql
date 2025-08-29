// use std::collections::{HashMap, HashSet};

// use prjunnamed_netlist::{Cell, CellRef, Design};

// use crate::{
//     node::{NodeSource, NodeType, net_to_source},
//     profiling::Timer,
// };

// #[derive(Clone, Debug)]
// pub struct DesignIndex {
//     design: Design,

//     // nodes: Vec<CellRef<'a>>,
//     /// Nodes of design in topological order (Name nodes filtered out)
//     nodes_topo: Vec<NodeId>,
//     by_type: HashMap<NodeType, Vec<NodeId>>,
//     /// For each driver (key), the list of (sink, sink_pin_idx)
//     reverse_node_lookup: HashMap<NodeId, Vec<(NodeId, usize)>>,

//     /// Pre-computed source information for each node: sources[sink][pin_idx] = NodeSource
//     node_sources: HashMap<NodeId, Vec<NodeSource>>,

//     /// fanout membership: driver -> (sink -> set_of_pins)
//     fanout_map: HashMap<NodeId, HashMap<NodeId, HashSet<usize>>>,

//     gate_count: usize,
// }

// impl DesignIndex {
//     pub fn build(design: Design) -> Self {
//         let _t = Timer::new("GraphIndex::build");

//         let mut by_type: HashMap<NodeType, Vec<NodeId>> = HashMap::new();

//         let nodes = design.iter_cells().collect::<Vec<_>>();

//         let nodes_ids: Vec<NodeId> = nodes.iter().map(|cell| NodeId::from(*cell)).collect();
//         debug_assert!(
//             !nodes_ids.is_empty(),
//             "DesignIndex: design has no cells (empty design?)"
//         );
//         debug_assert!(
//             nodes_ids.is_sorted(),
//             "DesignIndex: design has unsorted cells"
//         );

//         let nodes_topo: Vec<NodeId> = design
//             .iter_cells_topo()
//             .rev()
//             .filter(|cell_ref| {
//                 let node_type = NodeType::from(cell_ref.get().as_ref());
//                 !matches!(node_type, NodeType::Name)
//             })
//             .map(|cell_ref| NodeId::from(cell_ref))
//             .collect();

//         println!("GraphIndex: {} nodes (topo order)", nodes_topo.len());

//         for node_ref in nodes_topo.iter().cloned() {
//             let node_type = NodeType::from(node_ref.get_cell_ref(&nodes).get().as_ref());
//             by_type.entry(node_type).or_default().push(node_ref);
//         }

//         println!("GraphIndex: {} nodes (by type)", by_type.len());

//         let mut reverse_node_lookup: HashMap<NodeId, Vec<(NodeId, usize)>> = HashMap::new();
//         let mut node_sources: HashMap<NodeId, Vec<NodeSource>> = HashMap::new();
//         let mut fanout_map: HashMap<NodeId, HashMap<NodeId, HashSet<usize>>> = HashMap::new();

//         let gate_count = nodes_topo
//             .iter()
//             .filter(|c| NodeType::from(c.get_cell_ref(&nodes).get().as_ref()).is_logic_gate())
//             .count();

//         println!("GraphIndex: {} gates", gate_count);

//         // Pre-compute source information for all nodes (as sinks)
//         for node_id in nodes_topo.iter() {
//             let mut sources: Vec<NodeSource> = Vec::new();
//             node_id.get_cell_ref(&nodes).visit(|net| {
//                 sources.push(net_to_source(&design, net));
//             });
//             let num_sources = sources.len();
//             node_sources.insert(*node_id, sources);
//             println!(
//                 "GraphIndex: node #{} has {} sources",
//                 node_id.debug_index(), num_sources
//             );
//         }

//         // Build reverse lookups and O(1) fanout membership map
//         for sink_ref in nodes_topo.iter() {
//             let sources = node_sources.get(&sink_ref).unwrap();
//             for (sink_pin_idx, source) in sources.iter().enumerate() {
//                 let driver_node = match source {
//                     NodeSource::Gate(node_ref, _src_bit) => Some(*node_ref),
//                     NodeSource::Io(node_ref, _src_bit) => Some(*node_ref),
//                     NodeSource::Const(_trit) => None,
//                 };

//                 if let Some(driver) = driver_node {
//                     let driver_node = NodeId::from(driver);
//                     // reverse list (driver -> vec of (sink, pin))
//                     reverse_node_lookup
//                         .entry(driver_node)
//                         .or_default()
//                         .push((*sink_ref, sink_pin_idx));

//                     // fanout_map (driver -> map sink -> set_of_pins)
//                     let entry = fanout_map.entry(driver_node).or_default();
//                     entry
//                         .entry(*sink_ref)
//                         .or_default()
//                         .insert(sink_pin_idx);
//                 }
//             }
//         }

//         DesignIndex {
//             design,
//             nodes_topo,
//             by_type,
//             reverse_node_lookup,
//             node_sources,
//             fanout_map,
//             gate_count,
//         }
//     }

//     pub fn get_cell_ref(&self, node_id: NodeId) -> Option<CellRef<'_>> {
//         let cells = self.design.iter_cells().collect::<Vec<_>>();
//         // binary search
//         cells.binary_search_by_key(&node_id.debug_index(), |cell| cell.debug_index())
//             .map(|idx| cells[idx]).ok()
//     }

//     pub fn gate_count(&self) -> usize {
//         self.gate_count
//     }

//     pub fn get_by_type(&self, node_type: NodeType) -> &[NodeId] {
//         let _t = Timer::new("GraphIndex::get_by_type");
//         self.by_type
//             .get(&node_type)
//             .map(|v| v.as_slice())
//             .unwrap_or(&[])
//     }

//     pub fn get_nodes_topo(&self) -> &[NodeId] {
//         self.nodes_topo.as_slice()
//     }

//     pub fn get_fanouts(&self, node: NodeId) -> &[(NodeId, usize)] {
//         let _t = Timer::new("GraphIndex::get_fanouts");
//         let slice = self
//             .reverse_node_lookup
//             .get(&node)
//             .map(|v| v.as_slice())
//             .unwrap_or(&[]);
//         slice
//     }

//     pub fn get_node_sources(&self, node: NodeId) -> &[NodeSource] {
//         let _t = Timer::new("GraphIndex::get_node_sources");
//         self.node_sources
//             .get(&node)
//             .map(|v| v.as_slice())
//             .unwrap_or(&[])
//     }

//     pub fn get_input_name(&self, node: NodeId) -> Option<&str> {
//         let _t = Timer::new("GraphIndex::get_input_name");
//         let cell_ref = node.get_cell_ref(&self.design.iter_cells().collect());
//         match node.get() {
//             std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
//             _ => None,
//         }
//     }

//     pub fn get_output_name(&self, node: NodeId) -> Option<&'a str> {
//         let _t = Timer::new("GraphIndex::get_output_name");
//         match node.get() {
//             std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
//             _ => None,
//         }
//     }

//     pub fn node_summary(&self, node: NodeId) -> String {
//         let _t = Timer::new("GraphIndex::node_summary");
//         let node_type = NodeType::from(node.get().as_ref());
//         let iname = self.get_input_name(node).unwrap_or("");
//         let oname = self.get_output_name(node).unwrap_or("");
//         let n = if !iname.is_empty() { iname } else { oname };
//         if n.is_empty() {
//             format!("#{} {:?}", node.debug_index(), node_type)
//         } else {
//             format!("#{} {:?}({})", node.debug_index(), node_type, n)
//         }
//     }

//     /// True if `driver` has any fanout edge to `sink` (any input pin).
//     pub fn has_fanout_to(&self, driver: NodeId, sink: NodeId) -> bool {
//         let _t = Timer::new("GraphIndex::has_fanout_to");
//         self.fanout_map
//             .get(&driver)
//             .and_then(|m| m.get(&sink))
//             .is_some()
//     }

//     /// True if `driver` feeds `sink` specifically at `pin_idx`.
//     pub fn has_fanout_to_pin(&self, driver: NodeId, sink: NodeId, pin_idx: usize) -> bool {
//         let _t = Timer::new("GraphIndex::has_fanout_to_pin");
//         self.fanout_map
//             .get(&driver)
//             .and_then(|m| m.get(&sink))
//             .is_some_and(|pins| pins.contains(&pin_idx))
//     }

//     /// The unique driver of a given sink input pin, if any (Gate/Io sources only).
//     pub fn driver_of_sink_pin(&self, sink: NodeId, pin_idx: usize) -> Option<NodeId> {
//         let _t = Timer::new("GraphIndex::driver_of_sink_pin");
//         let src = self.get_node_sources(sink).get(pin_idx)?;
//         match src {
//             NodeSource::Gate(c, _) | NodeSource::Io(c, _) => Some(*c),
//             NodeSource::Const(_) => None,
//         }
//     }

//     /// All drivers of a sink across all pins (Gate/Io only), duplicates removed.
//     pub fn drivers_of_sink_all_pins(&self, sink: NodeId) -> Vec<NodeId> {
//         let _t = Timer::new("GraphIndex::drivers_of_sink_all_pins");
//         let mut out: Vec<NodeId> = self
//             .get_node_sources(sink)
//             .iter()
//             .filter_map(|src| match src {
//                 NodeSource::Gate(c, _) | NodeSource::Io(c, _) => Some(*c),
//                 NodeSource::Const(_) => None,
//             })
//             .collect();

//         out.sort_by_key(|c| c.debug_index());
//         out.dedup();
//         out
//     }
// }

// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct NodeId {
//     debug_index: usize,
// };

// impl NodeId {
//     pub fn new(debug_index: usize) -> Self {
//         NodeId { debug_index }
//     }
//     pub fn get_cell_ref<'a>(&self, cells: &Vec<CellRef<'a>>) -> CellRef<'a> {
//         cells[self.debug_index]
//     }
//     pub fn debug_index(&self) -> usize {
//         self.debug_index
//     }

// }

// impl From<CellRef<'_>> for NodeId {
//     fn from(c: CellRef<'_>) -> Self {
//         NodeId::new(c.debug_index())
//     }
// }
