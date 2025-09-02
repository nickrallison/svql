use std::collections::HashMap;

use crate::{Timer, cell::CellWrapper};

#[derive(Clone, Debug, Default)]
pub(super) struct CellMapping<'p, 'd> {
    /// Pattern to Design cell mapping
    pattern_to_design: HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    /// Design to Pattern cell mapping
    design_to_pattern: HashMap<CellWrapper<'d>, CellWrapper<'p>>,
}

impl<'p, 'd> CellMapping<'p, 'd> {
    #[contracts::debug_ensures(ret.pattern_to_design.is_empty())]
    #[contracts::debug_ensures(ret.design_to_pattern.is_empty())]
    pub(super) fn new() -> Self {
        Self {
            pattern_to_design: HashMap::new(),
            design_to_pattern: HashMap::new(),
        }
    }

    #[contracts::debug_requires(self.pattern_to_design.len() == self.design_to_pattern.len())]
    #[contracts::debug_ensures(self.pattern_to_design.len() == self.design_to_pattern.len())]
    pub(super) fn insert(&mut self, pattern: CellWrapper<'p>, design: CellWrapper<'d>) {
        let _t = Timer::new("NodeMapping::insert");

        self.pattern_to_design
            .insert(pattern.clone(), design.clone());
        self.design_to_pattern.insert(design, pattern);
    }

    // debug ensure that pattern & design mappings are consistent
    pub(super) fn get_design_cell(&self, pattern: CellWrapper<'p>) -> Option<CellWrapper<'d>> {
        let _t = Timer::new("NodeMapping::get_design_cell");
        let out = self.pattern_to_design.get(&pattern).cloned();
        out
    }

    // debug ensure that pattern & design mappings are consistent
    pub(super) fn get_pattern_cell(&self, design: CellWrapper<'d>) -> Option<CellWrapper<'p>> {
        let _t = Timer::new("NodeMapping::get_pattern_cell");
        let out = self.design_to_pattern.get(&design).cloned();
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
    pub(super) fn design_mapping(&self) -> &HashMap<CellWrapper<'d>, CellWrapper<'p>> {
        &self.design_to_pattern
    }

    #[contracts::debug_requires(self.pattern_to_design.len() == self.design_to_pattern.len())]
    pub(super) fn pattern_mapping(&self) -> &HashMap<CellWrapper<'p>, CellWrapper<'d>> {
        &self.pattern_to_design
    }

    pub(super) fn signature(&self) -> Vec<usize> {
        let _t = Timer::new("NodeMapping::signature");
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
