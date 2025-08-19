use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Cell;

use super::index::{Index, NodeId};
use crate::{
    cell_kind::CellWrapper,
    ports::{Source, is_commutative, normalize_commutative},
};

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
    pub(super) fn mappings(&self) -> &std::collections::HashMap<NodeId, NodeId> {
        &self.mapping
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

        // Name maps
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

        // Build (pattern Output bit) -> (design cell, bit) drivers
        let mut out_driver_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)> =
            HashMap::new();
        for oc in pat_output_cells {
            if let Cell::Output(_, value) = oc.cref.cref().get().as_ref() {
                for (out_bit, net) in value.iter().enumerate() {
                    if let Ok((p_src_cell_ref, p_bit)) = oc.cref.cref().design().find_cell(net) {
                        let p_src = CellWrapper::from(p_src_cell_ref);

                        if let Some(&d_src) = cell_mapping.get(&p_src) {
                            out_driver_map.insert((oc.cref, out_bit), (d_src, p_bit));
                            continue;
                        }

                        if let Some(&(d_cell, d_bit)) = self.boundary.get(&(p_src, p_bit)) {
                            out_driver_map.insert((oc.cref, out_bit), (d_cell, d_bit));
                        }
                    }
                }
            }
        }

        super::SubgraphMatch {
            cell_mapping,
            pat_input_cells: pat_input_cells.to_vec(),
            pat_output_cells: pat_output_cells.to_vec(),
            boundary_src_map,
            input_by_name,
            output_by_name,
            out_driver_map,
        }
    }
}

/// Compute deterministic aligned input pairs for pattern/design nodes,
/// respecting exact-length vs superset-length and commutativity normalization.
pub(super) fn aligned_sources<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    match_length: bool,
) -> Option<Vec<(Source<'p>, Source<'d>)>> {
    let kind = p_index.kind(p_id);

    let mut p_inputs = p_index.pins(p_id).inputs.clone();
    let mut d_inputs = d_index.pins(d_id).inputs.clone();

    if is_commutative(kind) {
        normalize_commutative(&mut p_inputs);
        normalize_commutative(&mut d_inputs);
    }

    let p_len = p_inputs.len();
    let d_len = d_inputs.len();

    if match_length && p_len != d_len {
        return None;
    }
    if !match_length && p_len > d_len {
        return None;
    }

    let take_len = std::cmp::min(p_len, d_len);

    let p_srcs = p_inputs.into_iter().map(|(_, s)| s);
    let d_srcs = d_inputs.into_iter().map(|(_, s)| s);

    Some(
        p_srcs
            .zip(d_srcs)
            .take(take_len)
            .map(|(p, d)| (p, d))
            .collect(),
    )
}

/// Validate aligned sources pairwise and collect any boundary insertions implied.
/// Does NOT mutate state; returns additions to apply if compatible.
pub(super) fn check_and_collect_boundary<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &State<'p, 'd>,
    match_length: bool,
) -> Option<Vec<((CellWrapper<'p>, usize), (CellWrapper<'d>, usize))>> {
    let pairs = aligned_sources(p_id, d_id, p_index, d_index, match_length)?;

    pairs.into_iter().try_fold(
        Vec::<((CellWrapper<'p>, usize), (CellWrapper<'d>, usize))>::new(),
        |mut additions, (p_src, d_src)| {
            use crate::ports::Source;

            match (p_src, d_src) {
                (Source::Const(pc), Source::Const(dc)) => {
                    if pc != dc {
                        return None;
                    }
                }
                (Source::Gate(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                    let Some(p_node) = p_index.try_cell_to_node(p_cell) else {
                        return None;
                    };
                    if let Some(mapped_d) = st.mapped_to(p_node) {
                        let Some(d_node) = d_index.try_cell_to_node(d_cell) else {
                            return None;
                        };
                        if mapped_d != d_node || p_bit != d_bit {
                            return None;
                        }
                    }
                }
                (Source::Io(p_cell, p_bit), Source::Gate(d_cell, d_bit))
                | (Source::Io(p_cell, p_bit), Source::Io(d_cell, d_bit)) => {
                    if let Some((exp_cell, exp_bit)) = st.boundary_get(p_cell, p_bit) {
                        if exp_cell != d_cell || exp_bit != d_bit {
                            return None;
                        }
                    } else {
                        additions.push(((p_cell, p_bit), (d_cell, d_bit)));
                    }
                }
                _ => return None,
            }

            Some(additions)
        },
    )
}
