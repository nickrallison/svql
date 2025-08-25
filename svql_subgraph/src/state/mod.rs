use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::Cell;

use crate::index::{Index, NodeId};
use crate::model::{CellWrapper, input_name, output_name};

mod align;
mod bindings;
mod constraints;

pub(crate) use bindings::{DesSrcKey, PatSrcKey, check_and_collect_bindings};
pub(crate) use constraints::{
    cells_compatible, design_has_input_from_bit, pattern_consumption_bits,
};

/// Self-documenting wrapper for an output source description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OutputSource<'p> {
    pub out_bit: usize,
    pub src_cell: CellWrapper<'p>,
    pub src_bit: usize,
}

pub(crate) struct State<'p, 'd> {
    // Pattern gate node -> Design gate node
    mapping: HashMap<NodeId, NodeId>,
    used_d: HashSet<NodeId>,

    // Unified driver bindings for External (IO) and possibly Const (if added later).
    bindings: HashMap<PatSrcKey<'p>, DesSrcKey<'d>>,

    target_gate_count: usize,
}

impl<'p, 'd> State<'p, 'd> {
    pub(crate) fn new(target_gate_count: usize) -> Self {
        State {
            mapping: HashMap::new(),
            used_d: HashSet::new(),
            bindings: HashMap::new(),
            target_gate_count,
        }
    }

    pub(crate) fn is_mapped(&self, p: NodeId) -> bool {
        self.mapping.contains_key(&p)
    }

    pub(crate) fn mapped_to(&self, p: NodeId) -> Option<NodeId> {
        self.mapping.get(&p).copied()
    }

    pub(crate) fn mappings(&self) -> &std::collections::HashMap<NodeId, NodeId> {
        &self.mapping
    }

    pub(crate) fn is_used_design(&self, d: NodeId) -> bool {
        self.used_d.contains(&d)
    }

    #[contracts::debug_requires(!self.is_mapped(p))]
    #[contracts::debug_requires(!self.is_used_design(d))]
    #[contracts::debug_ensures(self.is_mapped(p) && self.is_used_design(d))]
    pub(crate) fn map(&mut self, p: NodeId, d: NodeId) {
        self.mapping.insert(p, d);
        self.used_d.insert(d);
    }

    #[contracts::debug_requires(self.is_mapped(p) && self.is_used_design(d))]
    #[contracts::debug_ensures(!self.is_mapped(p) && !self.is_used_design(d))]
    pub(crate) fn unmap(&mut self, p: NodeId, d: NodeId) {
        self.mapping.remove(&p);
        self.used_d.remove(&d);
    }

    pub(crate) fn done(&self) -> bool {
        self.mapping.len() == self.target_gate_count
    }

    /// Get a previously established binding, if any.
    pub(crate) fn binding_get(&self, key: PatSrcKey<'p>) -> Option<DesSrcKey<'d>> {
        self.bindings.get(&key).copied()
    }

    /// Insert a new binding, returns true if inserted, false if it already existed.
    pub(crate) fn binding_insert(&mut self, key: PatSrcKey<'p>, val: DesSrcKey<'d>) -> bool {
        if self.bindings.contains_key(&key) {
            return false;
        }
        self.bindings.insert(key, val);
        true
    }

    /// Remove bindings for the provided keys (used for backtracking).
    #[contracts::debug_ensures(keys.iter().all(|k| self.binding_get(*k).is_none()))]
    pub(crate) fn bindings_remove_keys(&mut self, keys: &[PatSrcKey<'p>]) {
        for k in keys {
            self.bindings.remove(k);
        }
    }

    // We intentionally construct HashMaps keyed by CellWrapper here because the
    // public SubgraphMatch API exposes those keys. Suppress clippy’s warning.
    #[allow(clippy::mutable_key_type)]
    #[contracts::debug_ensures(ret.cell_mapping.len() == self.mapping.len())]
    pub(crate) fn to_subgraph_match(
        &self,
        p_index: &Index<'p>,
        d_index: &Index<'d>,
        pat_input_cells: &[CellWrapper<'p>],
        pat_output_cells: &[CellWrapper<'p>],
    ) -> crate::SubgraphMatch<'p, 'd> {
        let cell_mapping = self.build_cell_mapping(p_index, d_index);
        let input_binding_by_name = self.build_input_binding_by_name(d_index, pat_input_cells);
        let output_driver_by_name =
            self.build_output_driver_by_name(d_index, &cell_mapping, pat_output_cells);

        crate::SubgraphMatch {
            cell_mapping,
            input_binding_by_name,
            output_driver_by_name,
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

    /// Build pattern input name -> design cell mapping if uniquely determined by bindings.
    fn build_input_binding_by_name(
        &self,
        d_index: &Index<'d>,
        pat_input_cells: &[CellWrapper<'p>],
    ) -> HashMap<&'p str, CellWrapper<'d>> {
        // Collect per-pattern-input the set of design cells observed in bindings.
        let mut per_input: HashMap<CellWrapper<'p>, std::collections::HashSet<CellWrapper<'d>>> =
            HashMap::new();

        for (k, v) in &self.bindings {
            if let PatSrcKey::External { cell: p_cell, .. } = k {
                let d_cell = match v {
                    DesSrcKey::Gate { node, .. } => d_index.node_to_cell(*node),
                    DesSrcKey::External { cell, .. } => *cell,
                    DesSrcKey::Const(_) => continue,
                };
                per_input.entry(*p_cell).or_default().insert(d_cell);
            }
        }

        // Keep only inputs with a single unique design cell; map by name.
        let mut by_name: HashMap<&'p str, CellWrapper<'d>> = HashMap::new();
        for p_in in pat_input_cells {
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

    /// Build pattern output name -> design cell mapping if uniquely determined.
    #[allow(clippy::mutable_key_type)]
    fn build_output_driver_by_name(
        &self,
        d_index: &Index<'d>,
        cell_mapping: &HashMap<CellWrapper<'p>, CellWrapper<'d>>,
        pat_output_cells: &[CellWrapper<'p>],
    ) -> HashMap<&'p str, CellWrapper<'d>> {
        let mut by_name: HashMap<&'p str, CellWrapper<'d>> = HashMap::new();

        for oc in pat_output_cells {
            let Some(name) = output_name(oc) else {
                continue;
            };
            let drivers: std::collections::HashSet<CellWrapper<'d>> = self
                .output_sources_for_cell(oc)
                .into_iter()
                .filter_map(|os| {
                    self.resolve_design_driver_for_pattern_source(
                        d_index,
                        cell_mapping,
                        os.src_cell,
                        os.src_bit,
                    )
                    .map(|(d_cell, _)| d_cell)
                })
                .collect();

            if drivers.len() == 1 {
                if let Some(&only) = drivers.iter().next() {
                    by_name.insert(name, only);
                }
            }
        }

        by_name
    }

    /// Extract per-output sources in a self-documenting struct form.
    fn output_sources_for_cell(&self, oc: &CellWrapper<'p>) -> Vec<OutputSource<'p>> {
        match oc.cref().get().as_ref() {
            Cell::Output(_, value) => value
                .iter()
                .enumerate()
                .filter_map(|(out_bit, net)| {
                    oc.cref()
                        .design()
                        .find_cell(net)
                        .ok()
                        .map(|(p_src_cell_ref, p_bit)| OutputSource {
                            out_bit,
                            src_cell: CellWrapper::from(p_src_cell_ref),
                            src_bit: p_bit,
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
