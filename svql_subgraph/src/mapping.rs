use tracing::trace;
// svql_subgraph/src/mapping.rs
use prjunnamed_netlist::CellRef;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub(super) struct CellMapping<'p, 'd> {
    /// Pattern to Design cell mapping
    pattern_mapping: HashMap<CellRef<'p>, CellRef<'d>>,
    /// Design to Pattern cell mapping
    design_mapping: HashMap<CellRef<'d>, CellRef<'p>>,
}

impl<'p, 'd> CellMapping<'p, 'd> {
    #[contracts::debug_ensures(ret.pattern_mapping.len() == 0)]
    #[contracts::debug_ensures(ret.design_mapping.len() == 0)]
    pub(super) fn new() -> Self {
        tracing::event!(tracing::Level::TRACE, "CellMapping::new");
        Self {
            pattern_mapping: HashMap::new(),
            design_mapping: HashMap::new(),
        }
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    #[contracts::debug_ensures(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn insert(&mut self, pattern: CellRef<'p>, design: CellRef<'d>) {
        tracing::event!(
            tracing::Level::TRACE,
            "CellMapping::insert P#{} -> D#{} (before size={})",
            pattern.debug_index(),
            design.debug_index(),
            self.pattern_mapping.len()
        );
        self.pattern_mapping.insert(pattern, design);
        self.design_mapping.insert(design, pattern);
        tracing::event!(
            tracing::Level::TRACE,
            "CellMapping::insert done (after size={})",
            self.pattern_mapping.len()
        );
    }

    // debug ensure that pattern & design mappings are consistent
    pub(super) fn get_design_cell(&self, pattern: CellRef<'p>) -> Option<CellRef<'d>> {
        let out = self.pattern_mapping.get(&pattern).copied();
        tracing::event!(
            tracing::Level::TRACE,
            "CellMapping::get_design_cell P#{} -> {:?}",
            pattern.debug_index(),
            out.as_ref().map(|c| c.debug_index())
        );
        out
    }

    // debug ensure that pattern & design mappings are consistent
    pub(super) fn get_pattern_cell(&self, design: CellRef<'d>) -> Option<CellRef<'p>> {
        let out = self.design_mapping.get(&design).copied();
        tracing::event!(
            tracing::Level::TRACE,
            "CellMapping::get_pattern_cell D#{} -> {:?}",
            design.debug_index(),
            out.as_ref().map(|c| c.debug_index())
        );
        out
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn len(&self) -> usize {
        debug_assert_eq!(self.pattern_mapping.len(), self.design_mapping.len());
        self.pattern_mapping.len()
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn is_empty(&self) -> bool {
        debug_assert_eq!(self.pattern_mapping.len(), self.design_mapping.len());
        self.pattern_mapping.is_empty()
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn design_mapping(&self) -> &HashMap<CellRef<'d>, CellRef<'p>> {
        &self.design_mapping
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn pattern_mapping(&self) -> &HashMap<CellRef<'p>, CellRef<'d>> {
        &self.pattern_mapping
    }

    pub(super) fn sig(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .pattern_mapping
            .values()
            .map(|d| d.debug_index())
            .collect();
        sig.sort_unstable();
        sig.dedup();
        tracing::event!(tracing::Level::TRACE, "CellMapping::sig -> {:?}", sig);
        sig
    }
}
