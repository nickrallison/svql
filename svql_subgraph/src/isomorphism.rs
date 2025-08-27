use prjunnamed_netlist::CellRef;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub(super) struct NodeMapping<'p, 'd> {
    /// Pattern to Design node mapping
    pattern_to_design: HashMap<CellRef<'p>, CellRef<'d>>,
    /// Design to Pattern node mapping
    design_to_pattern: HashMap<CellRef<'d>, CellRef<'p>>,
}

impl<'p, 'd> NodeMapping<'p, 'd> {
    #[contracts::debug_ensures(ret.pattern_to_design.is_empty())]
    #[contracts::debug_ensures(ret.design_to_pattern.is_empty())]
    pub(super) fn new() -> Self {
        tracing::event!(tracing::Level::TRACE, "NodeMapping::new");
        Self {
            pattern_to_design: HashMap::new(),
            design_to_pattern: HashMap::new(),
        }
    }

    #[contracts::debug_requires(self.pattern_to_design.len() == self.design_to_pattern.len())]
    #[contracts::debug_ensures(self.pattern_to_design.len() == self.design_to_pattern.len())]
    pub(super) fn insert(&mut self, pattern: CellRef<'p>, design: CellRef<'d>) {
        tracing::event!(
            tracing::Level::TRACE,
            "NodeMapping::insert P#{} -> D#{} (before size={})",
            pattern.debug_index(),
            design.debug_index(),
            self.pattern_to_design.len()
        );
        self.pattern_to_design.insert(pattern, design);
        self.design_to_pattern.insert(design, pattern);
        tracing::event!(
            tracing::Level::TRACE,
            "NodeMapping::insert done (after size={})",
            self.pattern_to_design.len()
        );
    }

    // debug ensure that pattern & design mappings are consistent
    pub(super) fn get_design_node(&self, pattern: CellRef<'p>) -> Option<CellRef<'d>> {
        let out = self.pattern_to_design.get(&pattern).copied();
        tracing::event!(
            tracing::Level::TRACE,
            "NodeMapping::get_design_node P#{} -> {:?}",
            pattern.debug_index(),
            out.as_ref().map(|c| c.debug_index())
        );
        out
    }

    // debug ensure that pattern & design mappings are consistent
    pub(super) fn get_pattern_node(&self, design: CellRef<'d>) -> Option<CellRef<'p>> {
        let out = self.design_to_pattern.get(&design).copied();
        tracing::event!(
            tracing::Level::TRACE,
            "NodeMapping::get_pattern_node D#{} -> {:?}",
            design.debug_index(),
            out.as_ref().map(|c| c.debug_index())
        );
        out
    }

    #[contracts::debug_requires(self.pattern_to_design.len() == self.design_to_pattern.len())]
    pub(super) fn len(&self) -> usize {
        debug_assert_eq!(self.pattern_to_design.len(), self.design_to_pattern.len());
        self.pattern_to_design.len()
    }

    #[contracts::debug_requires(self.pattern_to_design.len() == self.design_to_pattern.len())]
    pub(super) fn is_empty(&self) -> bool {
        debug_assert_eq!(self.pattern_to_design.len(), self.design_to_pattern.len());
        self.pattern_to_design.is_empty()
    }

    #[contracts::debug_requires(self.pattern_to_design.len() == self.design_to_pattern.len())]
    pub(super) fn design_mapping(&self) -> &HashMap<CellRef<'d>, CellRef<'p>> {
        &self.design_to_pattern
    }

    #[contracts::debug_requires(self.pattern_to_design.len() == self.design_to_pattern.len())]
    pub(super) fn pattern_mapping(&self) -> &HashMap<CellRef<'p>, CellRef<'d>> {
        &self.pattern_to_design
    }

    pub(super) fn signature(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .pattern_to_design
            .values()
            .map(|d| d.debug_index())
            .collect();
        sig.sort_unstable();
        sig.dedup();
        tracing::event!(tracing::Level::TRACE, "NodeMapping::signature -> {:?}", sig);
        sig
    }
}
