use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::{Cell, Trit};

use super::index::{Index, NodeId};
use crate::cell::{
    CellWrapper, Source, input_name, is_commutative, normalize_commutative, output_name,
};

/// A canonical representation of a pattern driver bit used by some sink pin.
/// This unifies previous "boundary" IO and constants, and can represent gates
/// (but we do not record Gate->Gate bindings; we only validate them).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum PatSrcKey<'p> {
    #[allow(dead_code)]
    Gate {
        node: NodeId,
        bit: usize,
    },
    External {
        cell: CellWrapper<'p>,
        bit: usize,
    },
    #[allow(dead_code)]
    Const(Trit),
}

/// A canonical representation of a design driver bit used by some sink pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum DesSrcKey<'d> {
    Gate {
        node: NodeId,
        bit: usize,
    },
    External {
        cell: CellWrapper<'d>,
        bit: usize,
    },
    #[allow(dead_code)]
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
        let cell_mapping = self.build_cell_mapping(p_index, d_index);

        let boundary_src_map = self.build_boundary_src_map(d_index);

        let input_by_name = name_map(pat_input_cells, input_name);
        let output_by_name = name_map(pat_output_cells, output_name);

        let out_driver_map = self.build_out_driver_map(d_index, &cell_mapping, pat_output_cells);

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

    #[allow(clippy::mutable_key_type)]
    fn build_cell_mapping(
        &self,
        p_index: &Index<'p>,
        d_index: &Index<'d>,
    ) -> HashMap<CellWrapper<'p>, CellWrapper<'d>> {
        self.mapping
            .iter()
            .map(|(&p_node, &d_node)| (p_index.node_to_cell(p_node), d_index.node_to_cell(d_node)))
            .collect()
    }

    #[allow(clippy::mutable_key_type)]
    fn build_boundary_src_map(
        &self,
        d_index: &Index<'d>,
    ) -> HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)> {
        self.bindings
            .iter()
            .filter_map(|(k, v)| match k {
                PatSrcKey::External {
                    cell: p_cell,
                    bit: p_bit,
                } => match v {
                    DesSrcKey::Gate {
                        node: d_node,
                        bit: d_bit,
                    } => Some(((*p_cell, *p_bit), (d_index.node_to_cell(*d_node), *d_bit))),
                    DesSrcKey::External {
                        cell: d_cell,
                        bit: d_bit,
                    } => Some(((*p_cell, *p_bit), (*d_cell, *d_bit))),
                    DesSrcKey::Const(_) => None,
                },
                _ => None,
            })
            .collect()
    }

    #[allow(clippy::mutable_key_type)]
    fn build_out_driver_map(
        &self,
        d_index: &Index<'d>,
        cell_mapping: &HashMap<CellWrapper<'p>, CellWrapper<'d>>,
        pat_output_cells: &[CellWrapper<'p>],
    ) -> HashMap<(CellWrapper<'p>, usize), (CellWrapper<'d>, usize)> {
        pat_output_cells
            .iter()
            .flat_map(|oc| {
                self.output_sources_for_cell(oc).into_iter().filter_map(
                    move |(out_bit, p_src, p_bit)| {
                        self.resolve_design_driver_for_pattern_source(
                            d_index,
                            cell_mapping,
                            p_src,
                            p_bit,
                        )
                        .map(|d| ((*oc, out_bit), d))
                    },
                )
            })
            .collect()
    }

    /// Extract (out_bit, pattern_source_cell, pattern_source_bit) for a pattern output cell.
    fn output_sources_for_cell(
        &self,
        oc: &CellWrapper<'p>,
    ) -> Vec<(usize, CellWrapper<'p>, usize)> {
        match oc.cref().get().as_ref() {
            Cell::Output(_, value) => value
                .iter()
                .enumerate()
                .filter_map(|(out_bit, net)| {
                    oc.cref()
                        .design()
                        .find_cell(net)
                        .ok()
                        .map(|(p_src_cell_ref, p_bit)| {
                            (out_bit, CellWrapper::from(p_src_cell_ref), p_bit)
                        })
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Resolve a design driver for a given (pattern source cell, bit).
    /// Prefers a mapped gate driver; falls back to previously recorded external bindings.
    fn resolve_design_driver_for_pattern_source(
        &self,
        d_index: &Index<'d>,
        cell_mapping: &HashMap<CellWrapper<'p>, CellWrapper<'d>>,
        p_src: CellWrapper<'p>,
        p_bit: usize,
    ) -> Option<(CellWrapper<'d>, usize)> {
        if let Some(&d_src) = cell_mapping.get(&p_src) {
            return Some((d_src, p_bit));
        }

        let key = PatSrcKey::External {
            cell: p_src,
            bit: p_bit,
        };
        match self.binding_get(key) {
            Some(DesSrcKey::Gate { node, bit }) => Some((d_index.node_to_cell(node), bit)),
            Some(DesSrcKey::External { cell, bit }) => Some((cell, bit)),
            _ => None,
        }
    }
}

/// Build a map from IO name to CellWrapper, ignoring unnamed items.
fn name_map<'p>(
    cells: &[CellWrapper<'p>],
    namer: fn(&CellWrapper<'p>) -> Option<&'p str>,
) -> HashMap<&'p str, CellWrapper<'p>> {
    cells
        .iter()
        .filter_map(|c| namer(c).map(|nm| (nm, *c)))
        .collect()
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

    Some(p_inputs.into_iter().zip(d_inputs).take(take_len).collect())
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
    let mut additions: BindingAdditions<'p, 'd> = Vec::new();

    for (p_src, d_src) in pairs {
        match (p_src, d_src) {
            (Source::Const(pc), Source::Const(dc)) => {
                if pc != dc {
                    return None;
                }
            }
            (Source::Gate(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                let p_node = p_index.try_cell_to_node(p_cell)?;
                let d_node = d_index.try_cell_to_node(d_cell)?;
                if !mapped_gate_pair_ok(st, p_node, p_bit, d_node, d_bit) {
                    return None;
                }
            }
            (Source::Io(p_cell, p_bit), d_src @ (Source::Gate(_, _) | Source::Io(_, _))) => {
                let p_key = PatSrcKey::External {
                    cell: p_cell,
                    bit: p_bit,
                };
                let d_key = des_key_from_gate_or_io(d_index, d_src)?;
                if !unify_external_binding(st, &mut additions, p_key, d_key) {
                    return None;
                }
            }
            _ => return None,
        }
    }

    Some(additions)
}

fn mapped_gate_pair_ok<'p, 'd>(
    st: &State<'p, 'd>,
    p_node: NodeId,
    p_bit: usize,
    d_node: NodeId,
    d_bit: usize,
) -> bool {
    match st.mapped_to(p_node) {
        Some(mapped_d) => mapped_d == d_node && p_bit == d_bit,
        None => true,
    }
}

/// Convert a design-side Source (Gate or Io) into a DesSrcKey.
/// Returns None if the source is not supported in this context (e.g., Const).
fn des_key_from_gate_or_io<'d>(d_index: &Index<'d>, d_src: Source<'d>) -> Option<DesSrcKey<'d>> {
    match d_src {
        Source::Gate(d_cell, d_bit) => {
            let d_node = d_index.try_cell_to_node(d_cell)?;
            Some(DesSrcKey::Gate {
                node: d_node,
                bit: d_bit,
            })
        }
        Source::Io(d_cell, d_bit) => Some(DesSrcKey::External {
            cell: d_cell,
            bit: d_bit,
        }),
        _ => None,
    }
}

/// Insert-or-validate a binding for a pattern External source.
/// - If a binding exists, it must match d_key.
/// - If not, record it in additions (to be inserted by the caller later).
fn unify_external_binding<'p, 'd>(
    st: &State<'p, 'd>,
    additions: &mut BindingAdditions<'p, 'd>,
    p_key: PatSrcKey<'p>,
    d_key: DesSrcKey<'d>,
) -> bool {
    match st.binding_get(p_key) {
        Some(existing) => existing == d_key,
        None => {
            additions.push((p_key, d_key));
            true
        }
    }
}

pub(super) fn cells_compatible<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    state: &State<'p, 'd>,
    match_length: bool,
) -> bool {
    if p_index.kind(p_id) != d_index.kind(d_id) {
        return false;
    }

    check_and_collect_bindings(p_id, d_id, p_index, d_index, state, match_length)
        .is_some_and(|_| downstream_consumers_compatible(p_id, d_id, p_index, d_index, state))
}

/// Return all bit indices on q_p's inputs that are driven by p_id in the pattern.
fn pattern_consumption_bits<'p>(p_index: &Index<'p>, q_p: NodeId, p_id: NodeId) -> Vec<usize> {
    p_index
        .pins(q_p)
        .inputs
        .iter()
        .filter_map(|p_src| {
            let (p_src_cell, p_bit) = match p_src {
                Source::Gate(c, b) => (*c, *b),
                _ => return None,
            };
            let p_src_node = p_index.try_cell_to_node(p_src_cell)?;
            (p_src_node == p_id).then_some(p_bit)
        })
        .collect()
}

/// Does the mapped design consumer q_d have an input from d_id at the given bit?
fn design_has_input_from_bit<'d>(
    d_index: &Index<'d>,
    q_d: NodeId,
    d_id: NodeId,
    bit: usize,
) -> bool {
    d_index.pins(q_d).inputs.iter().any(|d_src| match d_src {
        Source::Gate(d_src_cell, d_bit) => d_index
            .try_cell_to_node(*d_src_cell)
            .is_some_and(|d_src_node| d_src_node == d_id && *d_bit == bit),
        _ => false,
    })
}

/// Ensure that for every already-mapped consumer (q_p -> q_d), any usage of p_id
/// as a source in q_p is mirrored by a usage of d_id in q_d at the same bit index.
///
/// This mirrors historical behavior and preserves automorphisms. It keeps matching
/// order-independent without recording producer-identity bindings.
fn downstream_consumers_compatible<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    state: &State<'p, 'd>,
) -> bool {
    state.mappings().iter().all(|(&q_p, &q_d)| {
        let required_bits = pattern_consumption_bits(p_index, q_p, p_id);
        required_bits
            .iter()
            .all(|&bit| design_has_input_from_bit(d_index, q_d, d_id, bit))
    })
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use crate::cell::CellKind;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::util::load_design_from("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v").unwrap();
    }

    #[test]
    fn same_cell_is_compatible_with_itself() {
        let d = &*SDFFE;
        let idx = super::Index::build(d);
        let st = super::State::<'_, '_>::new(idx.gate_count());
        let match_length = true;

        for &n in idx.of_kind(crate::cell::CellKind::Dff) {
            assert!(cells_compatible(n, n, &idx, &idx, &st, match_length));
        }
    }

    #[test]
    fn pattern_io_can_bind_to_design_gate() {
        let d_p = &SDFFE;
        let d_d = &SEQ_DOUBLE_SDFFE;

        let p_idx = super::Index::build(d_p);
        let d_idx = super::Index::build(d_d);
        let st = super::State::new(p_idx.gate_count());

        let match_length = true;

        let p = p_idx.of_kind(CellKind::Dff)[0];
        for &d in d_idx.of_kind(CellKind::Dff) {
            assert!(
                cells_compatible(p, d, &p_idx, &d_idx, &st, match_length),
                "pattern IO D should be compatible with design DFF regardless of external driver kind"
            );
        }
    }
}
