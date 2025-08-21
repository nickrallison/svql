use std::collections::{HashMap, HashSet};

use log::trace;
use prjunnamed_netlist::{Cell, CellHash, Design};

use crate::index::{Index, NodeId};
use crate::model::{CellWrapper, input_name, output_name};

mod align;
mod bindings;
mod constraints;

pub(crate) use bindings::{DesSrcKey, PatSrcKey, check_and_collect_bindings};
pub(crate) use constraints::cells_compatible;

/// Self-documenting wrapper for an output source description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OutputSource {
    pub out_bit: usize,
    pub src_cell: CellWrapper,
    pub src_bit: usize,
}

pub(crate) struct State<'p, 'd> {
    // Pattern gate node -> Design gate node
    mapping: HashMap<NodeId, NodeId>,
    used_d: HashSet<NodeId>,

    // Unified driver bindings for External (IO) and possibly Const (if added later).
    // Intentionally, we do NOT store Gate->Gate bindings here to avoid collapsing
    // automorphisms prematurely (keeps behavior identical to pre-refactor).
    bindings: HashMap<PatSrcKey, DesSrcKey>,

    target_gate_count: usize,
    p_design: &'p Design,
    d_design: &'d Design,
}

impl<'p, 'd> State<'p, 'd> {
    pub fn new(target_gate_count: usize, p_design: &'p Design, d_design: &'d Design) -> Self {
        trace!(
            "Creating new state with target gate count: {}",
            target_gate_count
        );
        State {
            mapping: HashMap::new(),
            used_d: HashSet::new(),
            bindings: HashMap::new(),
            target_gate_count,
            p_design,
            d_design,
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

    pub fn map(&mut self, p: NodeId, d: NodeId) {
        trace!("Mapping pattern {} -> design {}", p, d);
        self.mapping.insert(p, d);
        self.used_d.insert(d);
    }

    pub fn unmap(&mut self, p: NodeId, d: NodeId) {
        trace!("Unmapping pattern {} -> design {}", p, d);
        self.mapping.remove(&p);
        self.used_d.remove(&d);
    }

    pub fn done(&self) -> bool {
        let result = self.mapping.len() == self.target_gate_count;
        trace!(
            "State done check: {} == {} -> {}",
            self.mapping.len(),
            self.target_gate_count,
            result
        );
        result
    }

    /// Get a previously established binding, if any.
    pub(crate) fn binding_get(&self, key: PatSrcKey) -> Option<DesSrcKey> {
        self.bindings.get(&key).copied()
    }

    /// Insert a new binding, returns true if inserted, false if it already existed.
    /// Callers only pass External (IO) bindings here — we do not persist Gate->Gate.
    pub fn binding_insert(&mut self, key: PatSrcKey, val: DesSrcKey) -> bool {
        trace!("Inserting binding: {:?} -> {:?}", key, val);
        if self.bindings.contains_key(&key) {
            trace!("Binding already exists");
            return false;
        }
        self.bindings.insert(key, val);
        trace!("Binding inserted");
        true
    }

    /// Remove bindings for the provided keys (used for backtracking).
    pub(crate) fn bindings_remove_keys(&mut self, keys: &[PatSrcKey]) {
        for k in keys {
            self.bindings.remove(k);
        }
    }

    // We intentionally construct HashMaps keyed by CellWrapper here because the
    // public SubgraphMatch API exposes those keys. Suppress clippy's warning.
    #[allow(clippy::mutable_key_type)]
    pub fn to_subgraph_match(
        &self,
        p_index: &Index<'p>,
        d_index: &Index<'d>,
        pat_input_cells: &[CellWrapper],
        pat_output_cells: &[CellWrapper],
    ) -> crate::SubgraphMatch {
        trace!("Converting state to subgraph match");
        trace!("Current mappings: {:?}", self.mapping);
        trace!("Current bindings: {:?}", self.bindings);

        let cell_mapping = self.build_cell_mapping(p_index, d_index);
        trace!("Built cell mapping with {} entries", cell_mapping.len());

        let boundary_src_map = self.build_boundary_src_map(d_index);
        trace!(
            "Built boundary source map with {} entries",
            boundary_src_map.len()
        );

        let input_by_name = name_map(pat_input_cells, p_index.design(), |cell_wrapper, design| {
            input_name(cell_wrapper, design).ok().map(|s| s.to_string())
        });
        trace!("Built input name map with {} entries", input_by_name.len());

        let output_by_name = name_map(
            pat_output_cells,
            p_index.design(),
            |cell_wrapper, design| {
                output_name(cell_wrapper, design)
                    .ok()
                    .map(|s| s.to_string())
            },
        );
        trace!(
            "Built output name map with {} entries",
            output_by_name.len()
        );

        let out_driver_map = self.build_out_driver_map(d_index, &cell_mapping, pat_output_cells);
        trace!(
            "Built output driver map with {} entries",
            out_driver_map.len()
        );

        crate::SubgraphMatch {
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
        p_index: &Index,
        d_index: &Index,
    ) -> HashMap<CellWrapper, CellWrapper> {
        self.mapping
            .iter()
            .map(|(&p_node, &d_node)| (p_index.node_to_cell(p_node), d_index.node_to_cell(d_node)))
            .collect()
    }

    #[allow(clippy::mutable_key_type)]
    fn build_boundary_src_map(
        &self,
        d_index: &Index,
    ) -> HashMap<(CellWrapper, usize), (CellWrapper, usize)> {
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
        d_index: &Index,
        cell_mapping: &HashMap<CellWrapper, CellWrapper>,
        pat_output_cells: &[CellWrapper],
    ) -> HashMap<(CellWrapper, usize), (CellWrapper, usize)> {
        pat_output_cells
            .iter()
            .flat_map(|oc| {
                self.output_sources_for_cell(*oc)
                    .into_iter()
                    .filter_map(move |os| {
                        self.resolve_design_driver_for_pattern_source(
                            d_index,
                            cell_mapping,
                            os.src_cell,
                            os.src_bit,
                        )
                        .map(|d| ((*oc, os.out_bit), d))
                    })
            })
            .collect()
    }

    /// Extract per-output sources in a self-documenting struct form.
    fn output_sources_for_cell(
        &self,
        oc: CellWrapper,
        design: &Design,
    ) -> Result<Vec<OutputSource>, Box<dyn std::error::Error>> {
        let cell_ref = oc.try_into_valid_cell_wrapper(design)?;
        match cell_ref.get().as_ref() {
            Cell::Output(_, value) => value
                .iter()
                .enumerate()
                .filter_map(|(out_bit, net)| {
                    self.p_design
                        .find_cell(net)
                        .ok()
                        .map(|(p_src_cell_ref, p_bit)| {
                            let cell_hash =
                                CellHash::new(p_src_cell_ref.debug_index(), self.p_design_hash);
                            let src_cell_wrapper = CellWrapper::new(cell_hash);
                            OutputSource {
                                out_bit,
                                src_cell: src_cell_wrapper,
                                src_bit: p_bit,
                            }
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
        d_index: &Index,
        cell_mapping: &HashMap<CellWrapper, CellWrapper>,
        p_src: CellWrapper,
        p_bit: usize,
    ) -> Option<(CellWrapper, usize)> {
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
fn name_map(
    cells: &[CellWrapper],
    design: &Design,
    namer: fn(&CellWrapper, &Design) -> Option<String>,
) -> HashMap<String, CellWrapper> {
    cells
        .iter()
        .filter_map(|c| namer(c, design).map(|nm| (nm, *c)))
        .collect()
}

#[cfg(test)]
mod tests {
    use prjunnamed_netlist::Design;

    use crate::model::CellKind;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::test_support::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
        static ref SEQ_DOUBLE_SDFFE: Design = crate::test_support::load_design_from("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v").unwrap();
    }

    #[test]
    fn same_cell_is_compatible_with_itself() {
        let d = &*SDFFE;
        let idx = crate::index::Index::build(d);
        let st = super::State::new(idx.gate_count(), d, d);
        let match_length = true;

        for &n in idx.of_kind(CellKind::Dff) {
            assert!(cells_compatible(n, n, &idx, &idx, &st, match_length));
        }
    }

    #[test]
    fn pattern_io_can_bind_to_design_gate() {
        let d_p = &SDFFE;
        let d_d = &SEQ_DOUBLE_SDFFE;

        let p_idx = crate::index::Index::build(d_p);
        let d_idx = crate::index::Index::build(d_d);
        let st = super::State::new(p_idx.gate_count(), d_p, d_d);

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
