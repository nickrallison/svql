use std::collections::{HashMap, HashSet};

use super::index::{Index, NodeId};
use crate::subgraph::cell_kind::CellWrapper;

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

    pub(super) fn is_mapped(&self, p: NodeId) -> bool { self.mapping.contains_key(&p) }
    pub(super) fn mapped_to(&self, p: NodeId) -> Option<NodeId> { self.mapping.get(&p).copied() }
    pub(super) fn is_used_design(&self, d: NodeId) -> bool { self.used_d.contains(&d) }

    pub(super) fn map(&mut self, p: NodeId, d: NodeId) {
        self.mapping.insert(p, d);
        self.used_d.insert(d);
    }

    pub(super) fn unmap(&mut self, p: NodeId, d: NodeId) {
        self.mapping.remove(&p);
        self.used_d.remove(&d);
    }

    pub(super) fn boundary_get(&self, p_cell: CellWrapper<'p>, p_bit: usize) -> Option<(CellWrapper<'d>, usize)> {
        self.boundary.get(&(p_cell, p_bit)).copied()
    }
    pub(super) fn boundary_insert(&mut self, key: (CellWrapper<'p>, usize), val: (CellWrapper<'d>, usize)) -> bool {
        if self.boundary.contains_key(&key) { return false; }
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

        super::SubgraphMatch {
            cell_mapping,
            pat_input_cells: pat_input_cells.to_vec(),
            pat_output_cells: pat_output_cells.to_vec(),
            boundary_src_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read_input_to_design;
    use prjunnamed_netlist::Design;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = load_design_from("examples/patterns/basic/ff/sdffe.v");
    }

    fn load_design_from(path: &str) -> Design {
        read_input_to_design(None, path.to_string()).expect("Failed to read input design")
    }

    #[test]
    fn state_basic_map_unmap() {
        let d = &*SDFFE;
        let idx = Index::build(d);

        let mut st = State::new(idx.gate_count());
        let n = idx.of_kind(crate::subgraph::cell_kind::CellKind::Dff)[0];
        st.map(n, n);
        assert!(st.is_mapped(n));
        assert!(st.is_used_design(n));
        st.unmap(n, n);
        assert!(!st.is_mapped(n));
        assert!(!st.is_used_design(n));
    }
}