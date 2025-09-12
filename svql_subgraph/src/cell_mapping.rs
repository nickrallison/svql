use std::collections::HashMap;

use crate::{Timer, cell::CellWrapper};

#[derive(Clone, Debug, Default)]
pub struct CellMapping<'p, 'd> {
    /// Pattern to Design cell mapping
    pattern_to_design: HashMap<CellWrapper<'p>, CellWrapper<'d>>,
    /// Design to Pattern cell mapping
    design_to_pattern: HashMap<CellWrapper<'d>, CellWrapper<'p>>,
}

impl<'p, 'd> CellMapping<'p, 'd> {
    pub(super) fn new() -> Self {
        Self {
            pattern_to_design: HashMap::new(),
            design_to_pattern: HashMap::new(),
        }
    }

    pub(super) fn insert(&mut self, pattern: CellWrapper<'p>, design: CellWrapper<'d>) {
        let _t = Timer::new("NodeMapping::insert");

        self.pattern_to_design
            .insert(pattern.clone(), design.clone());
        self.design_to_pattern.insert(design, pattern);
    }

    pub(super) fn remove_by_pattern(
        &mut self,
        pattern: CellWrapper<'p>,
    ) -> Option<CellWrapper<'d>> {
        let _t = Timer::new("NodeMapping::remove_by_pattern");
        if let Some(design_cell) = self.pattern_to_design.remove(&pattern) {
            self.design_to_pattern.remove(&design_cell);
            return Some(design_cell);
        }
        None
    }

    // debug ensure that pattern & design mappings are consistent
    pub fn get_design_cell(&self, pattern: CellWrapper<'p>) -> Option<CellWrapper<'d>> {
        let _t = Timer::new("NodeMapping::get_design_cell");

        self.pattern_to_design.get(&pattern).cloned()
    }

    // debug ensure that pattern & design mappings are consistent
    pub fn get_pattern_cell(&self, design: CellWrapper<'d>) -> Option<CellWrapper<'p>> {
        let _t = Timer::new("NodeMapping::get_pattern_cell");

        self.design_to_pattern.get(&design).cloned()
    }

    pub fn len(&self) -> usize {
        debug_assert_eq!(self.pattern_to_design.len(), self.design_to_pattern.len());
        self.pattern_to_design.len()
    }

    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.pattern_to_design.len(), self.design_to_pattern.len());
        self.pattern_to_design.is_empty()
    }

    pub fn design_mapping(&self) -> &HashMap<CellWrapper<'d>, CellWrapper<'p>> {
        &self.design_to_pattern
    }

    pub fn pattern_mapping(&self) -> &HashMap<CellWrapper<'p>, CellWrapper<'d>> {
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
