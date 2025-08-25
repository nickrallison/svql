use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Design;

use crate::index::{Index, NodeId};
use crate::model::{CellWrapper, Source, input_name, output_name};
use crate::search::heuristics::rarest_gate_heuristic;
use crate::state::{design_has_input_from_bit, pattern_consumption_bits};
use svql_common::{Config, DedupeMode};

/// Lightweight state for the simplified (wildcard-IO) matcher.
struct StateSimple {
    mapping: HashMap<NodeId, NodeId>,
    used_d: HashSet<NodeId>,
    target_gate_count: usize,
}

impl StateSimple {
    fn new(target_gate_count: usize) -> Self {
        StateSimple {
            mapping: HashMap::new(),
            used_d: HashSet::new(),
            target_gate_count,
        }
    }

    fn is_mapped(&self, p: NodeId) -> bool {
        self.mapping.contains_key(&p)
    }

    fn mapped_to(&self, p: NodeId) -> Option<NodeId> {
        self.mapping.get(&p).copied()
    }

    fn is_used_design(&self, d: NodeId) -> bool {
        self.used_d.contains(&d)
    }

    fn map(&mut self, p: NodeId, d: NodeId) {
        self.mapping.insert(p, d);
        self.used_d.insert(d);
    }

    fn unmap(&mut self, p: NodeId, d: NodeId) {
        self.mapping.remove(&p);
        self.used_d.remove(&d);
    }

    fn done(&self) -> bool {
        self.mapping.len() == self.target_gate_count
    }
}

/// Public entry for the simplified mode (wildcard pattern inputs).
pub(crate) fn find_subgraphs_simple<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
    config: &Config,
) -> crate::AllSubgraphMatches<'p, 'd> {
    let p_index = Index::build(pattern);
    let d_index = Index::build(design);

    if p_index.gate_count() == 0 || d_index.gate_count() == 0 {
        return crate::AllSubgraphMatches { matches: vec![] };
    }

    let Some(anchor) = rarest_gate_heuristic(&p_index, &d_index) else {
        return crate::AllSubgraphMatches { matches: vec![] };
    };

    let (pat_inputs, pat_outputs) = crate::get_pattern_io_cells(pattern);

    // Deterministic pattern anchor (minimum NodeId).
    let p_anchor = *anchor
        .pat_anchors
        .iter()
        .min()
        .expect("No pattern anchors found");

    let mut results: Vec<crate::SubgraphMatch<'p, 'd>> = Vec::new();

    for &d_anchor in &anchor.des_anchors {
        if !cells_compatible_simple(
            p_anchor,
            d_anchor,
            &p_index,
            &d_index,
            &StateSimple::new(p_index.gate_count()),
            config.match_length,
        ) {
            continue;
        }

        let mut st = StateSimple::new(p_index.gate_count());
        st.map(p_anchor, d_anchor);

        backtrack_simple(
            &p_index,
            &d_index,
            &mut st,
            &mut results,
            &pat_inputs,
            &pat_outputs,
            config,
        );

        st.unmap(p_anchor, d_anchor);
    }

    // Deduplication per config.
    match config.dedupe {
        DedupeMode::None => {
            // keep all raw matches
        }
        DedupeMode::AutoMorph => {
            let mut seen: HashSet<Vec<usize>> = HashSet::new();
            results.retain(|m| seen.insert(super::dedupe::signature_mapped_gate_set(m)));
        }
    }

    crate::AllSubgraphMatches { matches: results }
}

fn backtrack_simple<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &mut StateSimple,
    out: &mut Vec<crate::SubgraphMatch<'p, 'd>>,
    pat_inputs: &[CellWrapper<'p>],
    pat_outputs: &[CellWrapper<'p>],
    config: &Config,
) {
    if st.done() {
        out.push(to_subgraph_match_simple(
            st,
            p_index,
            d_index,
            pat_inputs,
            pat_outputs,
            config.match_length,
        ));
        return;
    }

    let Some(next_p) = choose_next_strict(p_index, st) else {
        return;
    };
    let kind = p_index.kind(next_p);

    let candidates: Vec<NodeId> = d_index
        .of_kind(kind)
        .iter()
        .copied()
        .filter(|d| !st.is_used_design(*d))
        .filter(|&d| cells_compatible_simple(next_p, d, p_index, d_index, st, config.match_length))
        .collect();

    for d_cand in candidates {
        st.map(next_p, d_cand);
        backtrack_simple(p_index, d_index, st, out, pat_inputs, pat_outputs, config);
        st.unmap(next_p, d_cand);
    }
}

/// Strict "inputs-resolved" selection: only pick pattern nodes whose gate inputs are all mapped.
fn choose_next_strict<'p, 'd>(p_index: &'p Index<'p>, st: &StateSimple) -> Option<NodeId> {
    (0..p_index.gate_count() as u32)
        .map(|i| i as NodeId)
        .find(|&p| !st.is_mapped(p) && inputs_resolved_for(p_index, st, p))
}

/// Are all gate inputs of `p` already mapped?
fn inputs_resolved_for<'p>(p_index: &'p Index<'p>, st: &StateSimple, p: NodeId) -> bool {
    p_index.pins(p).inputs.iter().all(|src| match src {
        Source::Const(_) => true,
        Source::Io(_, _) => true,
        Source::Gate(gc, _) => p_index
            .try_cell_to_node(*gc)
            .is_some_and(|g| st.is_mapped(g)),
    })
}

/// Align sources for a candidate pair (respects commutativity and arity policy).
fn aligned_sources_simple<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    match_length: bool,
) -> Option<Vec<(Source<'p>, Source<'d>)>> {
    let kind = p_index.kind(p_id);

    let mut p_inputs = p_index.pins(p_id).inputs.clone();
    let mut d_inputs = d_index.pins(d_id).inputs.clone();

    if crate::model::normalize::is_commutative(kind) {
        crate::model::normalize::normalize_commutative(&mut p_inputs);
        crate::model::normalize::normalize_commutative(&mut d_inputs);
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

    Some(
        p_inputs
            .into_iter()
            .zip(d_inputs.into_iter())
            .take(take_len)
            .collect(),
    )
}

/// Simplified compatibility:
/// - same kind
/// - align inputs
/// - Gate inputs preserve mapped producers (node and bit)
/// - IO inputs are wildcards; consts must match identically
/// - plus downstream-consumer adjacency compatibility with already mapped consumers
fn cells_compatible_simple<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    st: &StateSimple,
    match_length: bool,
) -> bool {
    if p_index.kind(p_id) != d_index.kind(d_id) {
        return false;
    }

    let Some(pairs) = aligned_sources_simple(p_id, d_id, p_index, d_index, match_length) else {
        return false;
    };

    for (p_src, d_src) in pairs {
        match (p_src, d_src) {
            (Source::Const(pc), Source::Const(dc)) => {
                if pc != dc {
                    return false;
                }
            }
            (Source::Const(_), _) => return false,
            (Source::Io(_, _), _) => {
                // wildcard: pattern Input can bind to any design source (IO/Gate/Const)
            }
            (Source::Gate(p_cell, p_bit), Source::Gate(d_cell, d_bit)) => {
                let Some(p_node) = p_index.try_cell_to_node(p_cell) else {
                    return false;
                };
                let Some(d_node) = d_index.try_cell_to_node(d_cell) else {
                    return false;
                };
                if let Some(mapped) = st.mapped_to(p_node) {
                    if mapped != d_node || p_bit != d_bit {
                        return false;
                    }
                }
            }
            (Source::Gate(_, _), _) => {
                // pattern gate input must be fed by a design gate to preserve gate graph edges
                return false;
            }
        }
    }

    downstream_consumers_compatible_simple(p_id, d_id, p_index, d_index, &st.mapping)
}

/// Ensure any already-mapped consumer q_p of p_id that uses bit `b` is mirrored in the design.
fn downstream_consumers_compatible_simple<'p, 'd>(
    p_id: NodeId,
    d_id: NodeId,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    mapping: &HashMap<NodeId, NodeId>,
) -> bool {
    mapping.iter().all(|(&q_p, &q_d)| {
        let required_bits = pattern_consumption_bits(p_index, q_p, p_id);
        required_bits
            .iter()
            .all(|&bit| design_has_input_from_bit(d_index, q_d, d_id, bit))
    })
}

/// Build a SubgraphMatch with gate mapping and unique IO name bindings.
fn to_subgraph_match_simple<'p, 'd>(
    st: &StateSimple,
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    pat_inputs: &[CellWrapper<'p>],
    pat_outputs: &[CellWrapper<'p>],
    match_length: bool,
) -> crate::SubgraphMatch<'p, 'd> {
    // Gate mapping
    let cell_mapping: HashMap<CellWrapper<'p>, CellWrapper<'d>> = st
        .mapping
        .iter()
        .map(|(&p_node, &d_node)| (p_index.node_to_cell(p_node), d_index.node_to_cell(d_node)))
        .collect();

    // Input bindings by scanning aligned inputs for all mapped pairs; keep only unique
    let input_binding_by_name =
        build_input_binding_by_name_simple(p_index, d_index, &st.mapping, pat_inputs, match_length);

    // Output driver by name (from pattern outputs’ sources, resolved via cell_mapping)
    let output_driver_by_name = build_output_driver_by_name_simple(&cell_mapping, pat_outputs);

    crate::SubgraphMatch {
        cell_mapping,
        input_binding_by_name,
        output_driver_by_name,
    }
}

fn build_input_binding_by_name_simple<'p, 'd>(
    p_index: &Index<'p>,
    d_index: &Index<'d>,
    mapping: &HashMap<NodeId, NodeId>,
    pat_inputs: &[CellWrapper<'p>],
    match_length: bool,
) -> HashMap<&'p str, CellWrapper<'d>> {
    // Collect per pattern input cell the set of seen design cells.
    let mut per_input: HashMap<CellWrapper<'p>, HashSet<CellWrapper<'d>>> = HashMap::new();

    for (&p_node, &d_node) in mapping {
        if let Some(pairs) = aligned_sources_simple(p_node, d_node, p_index, d_index, match_length)
        {
            for (p_src, d_src) in pairs {
                if let Source::Io(p_cell, _) = p_src {
                    let d_cell_opt = match d_src {
                        Source::Gate(d_cell, _) => d_index
                            .try_cell_to_node(d_cell)
                            .map(|dn| d_index.node_to_cell(dn)),
                        Source::Io(d_cell, _) => Some(d_cell),
                        Source::Const(_) => None,
                    };
                    if let Some(d_cell) = d_cell_opt {
                        per_input.entry(p_cell).or_default().insert(d_cell);
                    }
                }
            }
        }
    }

    // Reduce to unique design cell per named pattern input
    let mut by_name: HashMap<&'p str, CellWrapper<'d>> = HashMap::new();
    for p_in in pat_inputs {
        if let Some(name) = input_name(p_in) {
            if let Some(set) = per_input.get(p_in) {
                if set.len() == 1 {
                    if let Some(&only) = set.iter().next() {
                        by_name.insert(name, only);
                    }
                }
            }
        }
    }
    by_name
}

fn build_output_driver_by_name_simple<'p, 'd>(
    cell_mapping: &HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    pat_outputs: &[CellWrapper<'p>],
) -> HashMap<&'p str, CellWrapper<'d>> {
    use prjunnamed_netlist::Cell;

    // For each output, find its source cell(s); if any source cell is a gate and mapped, use that design cell.
    let mut by_name: HashMap<&'p str, CellWrapper<'d>> = HashMap::new();

    for oc in pat_outputs {
        let Some(name) = output_name(oc) else {
            continue;
        };
        let mut set: HashSet<CellWrapper<'d>> = HashSet::new();

        if let Cell::Output(_, value) = oc.cref().get().as_ref() {
            for net in value {
                if let Ok((p_src_cell_ref, _bit)) = oc.cref().design().find_cell(net) {
                    let p_src = CellWrapper::from(p_src_cell_ref);
                    if let Some(&d_cell) = cell_mapping.get(&p_src) {
                        set.insert(d_cell);
                    }
                }
            }
        }

        if set.len() == 1 {
            if let Some(&only) = set.iter().next() {
                by_name.insert(name, only);
            }
        }
    }

    by_name
}
