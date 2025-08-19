use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Cell;

use super::index::{Index, NodeId};
use crate::cell_kind::CellWrapper;

pub(super) struct State<'p, 'd> {
    mapping: HashMap<NodeId, NodeId>,
    used_d: HashSet<NodeId>,
    boundary: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)>,
    target_gate_count: usize,
}

impl<'p, 'd> State<'p, 'd> {
    pub(super) fn new(target_gate_count: usize) -> Self {
        State {
            mapping: HashMap::new(),
            used_d: HashSet::new(),
            boundary: HashMap::new(),
            target_gate_count,
        }
    }

    pub(super) fn is_mapped(&self, p: NodeId) -> bool {
        self.mapping.contains_key(&p)
    }
    pub(super) fn mapped_to(&self, p: NodeId) -> Option<NodeId> {
        self.mapping.get(&p).copied()
    }
    pub(super) fn is_used_design(&self, d: NodeId) -> bool {
        self.used_d.contains(&d)
    }

    pub(super) fn map(&mut self, p: NodeId, d: NodeId) {
        self.mapping.insert(p, d);
        self.used_d.insert(d);
    }

    pub(super) fn unmap(&mut self, p: NodeId, d: NodeId) {
        self.mapping.remove(&p);
        self.used_d.remove(&d);
    }

    pub(super) fn boundary_get(
        &self,
        p_cell: CellWrapper<'p>,
        p_bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        self.boundary.get(&(p_cell, p_bit)).copied()
    }
    pub(super) fn boundary_insert(
        &mut self,
        key: (CellWrapper<'p>, usize),
        val: (CellWrapper<'d>, usize),
    ) -> bool {
        if self.boundary.contains_key(&key) {
            return false;
        }
        self.boundary.insert(key, val);
        true
    }
    pub(super) fn boundary_remove_keys(&mut self, keys: &[(CellWrapper<'p>, usize)]) {
        for k in keys {
            self.boundary.remove(k);
        }
    }

    pub(super) fn done(&self) -> bool {
        self.mapping.len() == self.target_gate_count
    }

    pub(super) fn to_subgraph_match(
        &self,
        p_index: &Index<'p>,
        d_index: &Index<'d>,
        pat_input_cells: &[super::cell_kind::InputCell<'p>],
        pat_output_cells: &[super::cell_kind::OutputCell<'p>],
    ) -> super::SubgraphMatch<'p, 'd> {
        let mut cell_mapping = HashMap::new();
        for (&p_node, &d_node) in &self.mapping {
            let p_cell = p_index.node_to_cell(p_node);
            let d_cell = d_index.node_to_cell(d_node);
            cell_mapping.insert(p_cell, d_cell);
        }

        let mut boundary_src_map = HashMap::new();
        for ((p_cell, p_bit), (d_cell, d_bit)) in &self.boundary {
            boundary_src_map.insert((*p_cell, *p_bit), (*d_cell, *d_bit));
        }

        // NEW: name maps
        let mut input_by_name = HashMap::new();
        for ic in pat_input_cells {
            if let Some(nm) = ic.name() {
                input_by_name.insert(nm, ic.cref);
            }
        }
        let mut output_by_name = HashMap::new();
        for oc in pat_output_cells {
            if let Some(nm) = oc.name() {
                output_by_name.insert(nm, oc.cref);
            }
        }

        // NEW: build (pattern Output bit) -> (design cell, bit) drivers
        let mut out_driver_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)> =
            HashMap::new();
        for oc in pat_output_cells {
            // Safely match the Output cell and pull its input Value
            if let Cell::Output(_, value) = oc.cref.cref().get().as_ref() {
                for (out_bit, net) in value.iter().enumerate() {
                    // Who drives this bit in the pattern?
                    if let Ok((p_src_cell_ref, p_bit)) = oc.cref.cref().design().find_cell(net) {
                        let p_src = CellWrapper::from(p_src_cell_ref);

                        // Prefer mapped gate
                        if let Some(&d_src) = cell_mapping.get(&p_src) {
                            out_driver_map.insert((oc.cref, out_bit), (d_src, p_bit));
                            continue;
                        }

                        // Fallback: boundary (IO-to-gate or IO-to-IO)
                        if let Some(&(d_cell, d_bit)) = self.boundary.get(&(p_src, p_bit)) {
                            out_driver_map.insert((oc.cref, out_bit), (d_cell, d_bit));
                        }
                        // else: constants/undef or unmapped sources -> no entry
                    }
                }
            }
        }

        super::SubgraphMatch {
            cell_mapping,
            pat_input_cells: pat_input_cells.to_vec(),
            pat_output_cells: pat_output_cells.to_vec(),
            boundary_src_map,
            input_by_name,  // NEW
            output_by_name, // NEW
            out_driver_map, // NEW
        }
    }
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn state_basic_map_unmap() {
        let d = &SDFFE;
        let idx = Index::build(d);

        let mut st = State::new(idx.gate_count());
        let n = idx.of_kind(crate::cell_kind::CellKind::Dff)[0];
        st.map(n, n);
        assert!(st.is_mapped(n));
        assert!(st.is_used_design(n));
        st.unmap(n, n);
        assert!(!st.is_mapped(n));
        assert!(!st.is_used_design(n));
    }
}
