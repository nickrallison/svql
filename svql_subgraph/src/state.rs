use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::{Cell, Trit};

use super::index::{Index, NodeId};
use crate::{
    cell::{CellWrapper, input_name, output_name},
    ports::{Source, is_commutative, normalize_commutative},
};

/// A canonical representation of a pattern driver bit used by some sink pin.
/// This unifies previous "boundary" IO and constants, and can represent gates
/// (but we do not record Gate->Gate bindings; we only validate them).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum PatSrcKey<'p> {
    Gate { node: NodeId, bit: usize },
    External { cell: CellWrapper<'p>, bit: usize },
    Const(Trit),
}

/// A canonical representation of a design driver bit used by some sink pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum DesSrcKey<'d> {
    Gate { node: NodeId, bit: usize },
    External { cell: CellWrapper<'d>, bit: usize },
    Const(Trit),
}

// Alias to keep signatures short and clear (satisfy clippy::type_complexity)
pub(super) type BindingAdditions<'p, 'd> = Vec<(PatSrcKey<'p>, DesSrcKey<'d>)>;

pub(super) struct State<'p, 'd> {
    // Pattern gate node -> Design gate node
    mapping: HashMap<NodeId, NodeId>,
    used_d: HashSet<NodeId>,

    // Unified driver bindings for External (IO) and possibly Const (if added later).
    // Intentionally, we do NOT store Gate->Gate bindings here to avoid collapsing
    // automorphisms prematurely (keeps behavior identical to pre-refactor).
    bindings: HashMap<PatSrcKey<'p>, DesSrcKey<'d>>,

    target_gate_count: usize,
}

impl<'p, 'd> State<'p, 'd> {
    pub(super) fn new(target_gate_count: usize) -> Self {
        State {
            mapping: HashMap::new(),
            used_d: HashSet::new(),
            bindings: HashMap::new(),
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

    pub(super) fn done(&self) -> bool {
        self.mapping.len() == self.target_gate_count
    }

    /// Get a previously established binding, if any.
    pub(super) fn binding_get(&self, key: PatSrcKey<'p>) -> Option<DesSrcKey<'d>> {
        self.bindings.get(&key).copied()
    }

    /// Insert a new binding, returns true if inserted, false if it already existed.
    /// Callers only pass External (IO) bindings here — we do not persist Gate->Gate.
    pub(super) fn binding_insert(&mut self, key: PatSrcKey<'p>, val: DesSrcKey<'d>) -> bool {
        if self.bindings.contains_key(&key) {
            return false;
        }
        self.bindings.insert(key, val);
        true
    }

    /// Remove bindings for the provided keys (used for backtracking).
    pub(super) fn bindings_remove_keys(&mut self, keys: &[PatSrcKey<'p>]) {
        for k in keys {
            self.bindings.remove(k);
        }
    }

    // We intentionally construct HashMaps keyed by CellWrapper here because the
    // public SubgraphMatch API exposes those keys. Suppress clippy’s warning.
    #[allow(clippy::mutable_key_type)]
    pub(super) fn to_subgraph_match(
        &self,
        p_index: &Index<'p>,
        d_index: &Index<'d>,
        pat_input_cells: &[CellWrapper<'p>],
        pat_output_cells: &[CellWrapper<'p>],
    ) -> super::SubgraphMatch<'p, 'd> {
        // Gate mapping (pattern gate -> design gate)
        let mut cell_mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>> = HashMap::new();
        for (&p_node, &d_node) in &self.mapping {
            let p_cell = p_index.node_to_cell(p_node);
            let d_cell = d_index.node_to_cell(d_node);
            cell_mapping.insert(p_cell, d_cell);
        }

        // Keep public boundary semantics: only expose bindings for pattern External IO keys.
        let mut boundary_src_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)> =
            HashMap::new();
        for (k, v) in &self.bindings {
            if let PatSrcKey::External {
                cell: p_cell,
                bit: p_bit,
            } = k
            {
                match v {
                    DesSrcKey::Gate {
                        node: d_node,
                        bit: d_bit,
                    } => {
                        let d_cell = d_index.node_to_cell(*d_node);
                        boundary_src_map.insert((*p_cell, *p_bit), (d_cell, *d_bit));
                    }
                    DesSrcKey::External {
                        cell: d_cell,
                        bit: d_bit,
                    } => {
                        boundary_src_map.insert((*p_cell, *p_bit), (*d_cell, *d_bit));
                    }
                    DesSrcKey::Const(_) => {
                        // Not surfaced in boundary_src_map (can be added later if desired).
                    }
                }
            }
        }

        // Name → pattern IO lookup
        let mut input_by_name = HashMap::new();
        for ic in pat_input_cells {
            if let Some(nm) = input_name(ic) {
                input_by_name.insert(nm, *ic);
            }
        }

        let mut output_by_name = HashMap::new();
        for oc in pat_output_cells {
            if let Some(nm) = output_name(oc) {
                output_by_name.insert(nm, *oc);
            }
        }

        // For each pattern output bit, find design driver:
        // 1) If it is driven by a pattern gate: use cell_mapping for the gate.
        // 2) Else (driven by a pattern external): use bindings to resolve the design driver.
        let mut out_driver_map: HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)> =
            HashMap::new();

        for oc in pat_output_cells {
            if let Cell::Output(_, value) = oc.cref().get().as_ref() {
                for (out_bit, net) in value.iter().enumerate() {
                    if let Ok((p_src_cell_ref, p_bit)) = oc.cref().design().find_cell(net) {
                        let p_src = CellWrapper::from(p_src_cell_ref);

                        if let Some(&d_src) = cell_mapping.get(&p_src) {
                            out_driver_map.insert((*oc, out_bit), (d_src, p_bit));
                            continue;
                        }

                        // Reconstruct External binding for this driver (if present)
                        let key = PatSrcKey::External {
                            cell: p_src,
                            bit: p_bit,
                        };
                        if let Some(v) = self.binding_get(key) {
                            match v {
                                DesSrcKey::Gate {
                                    node: d_node,
                                    bit: d_bit,
                                } => {
                                    out_driver_map.insert(
                                        (*oc, out_bit),
                                        (d_index.node_to_cell(d_node), d_bit),
                                    );
                                }
                                DesSrcKey::External {
                                    cell: d_cell,
                                    bit: d_bit,
                                } => {
                                    out_driver_map.insert((*oc, out_bit), (d_cell, d_bit));
                                }
                                DesSrcKey::Const(_) => {
                                    // Skip: not surfaced currently.
                                }
                            }
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

    let p_srcs = p_inputs.into_iter();
    let d_srcs = d_inputs.into_iter();

    Some(p_srcs.zip(d_srcs).take(take_len).collect())
}

/// Validate aligned sources pairwise and collect any driver bindings implied.
/// Does NOT mutate state; returns additions to apply if compatible.
///
/// Unification rule:
/// - IO and Const: we record (or validate) bindings.
/// - Gate: we only validate if already mapped; we do NOT record bindings to avoid
///   collapsing automorphisms prematurely.
pub(super) fn check_and_collect_bindings<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &State<'p, 'd>,
    match_length: bool,
) -> Option<BindingAdditions<'p, 'd>> {
    let pairs = aligned_sources(p_id, d_id, p_index, d_index, match_length)?;

    pairs.into_iter().try_fold(
        Vec::<(PatSrcKey<'p>, DesSrcKey<'d>)>::new(),
        |mut additions, (p_src, d_src)| {
            match (p_src, d_src) {
                (Source::Const(pc), Source::Const(dc)) => {
                    if pc != dc {
                        return None;
                    }
                    // Const-Const is validated; not recorded.
                }
                (Source::Gate(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                    let p_node = p_index.try_cell_to_node(p_cell)?;
                    let d_node = d_index.try_cell_to_node(d_cell)?;
                    // If producer already mapped, it must equal the aligned design node,
                    // and the bit must match.
                    if let Some(mapped_d) = st.mapped_to(p_node) {
                        if mapped_d != d_node || p_bit != d_bit {
                            return None;
                        }
                    }
                    // Do NOT record Gate->Gate bindings here.
                }
                (Source::Io(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                    let d_node = d_index.try_cell_to_node(d_cell)?;
                    let p_key = PatSrcKey::External {
                        cell: p_cell,
                        bit: p_bit,
                    };
                    let d_key = DesSrcKey::Gate {
                        node: d_node,
                        bit: d_bit,
                    };
                    if let Some(existing) = st.binding_get(p_key) {
                        if existing != d_key {
                            return None;
                        }
                    } else {
                        additions.push((p_key, d_key));
                    }
                }
                (Source::Io(p_cell, p_bit), Source::Io(d_cell, d_bit)) => {
                    let p_key = PatSrcKey::External {
                        cell: p_cell,
                        bit: p_bit,
                    };
                    let d_key = DesSrcKey::External {
                        cell: d_cell,
                        bit: d_bit,
                    };
                    if let Some(existing) = st.binding_get(p_key) {
                        if existing != d_key {
                            return None;
                        }
                    } else {
                        additions.push((p_key, d_key));
                    }
                }
                // Extend here if you want IO to match constants in the future:
                // (Source::Io(p_cell, p_bit), Source::Const(dc)) => { ... }
                _ => return None,
            }
            Some(additions)
        },
    )
}
