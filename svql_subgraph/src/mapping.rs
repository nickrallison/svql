use std::collections::HashMap;

use crate::CellRefWrapper;

#[derive(Clone, Debug, Default)]
pub(super) struct CellMapping<'p, 'd> {
    /// Pattern to Design cell mapping
    pattern_mapping: HashMap<CellRefWrapper<'p>, CellRefWrapper<'d>>,
    /// Design to Pattern cell mapping
    design_mapping: HashMap<CellRefWrapper<'d>, CellRefWrapper<'p>>,
}

impl<'p, 'd> CellMapping<'p, 'd> {
    #[contracts::debug_ensures(ret.pattern_mapping.len() == 0)]
    #[contracts::debug_ensures(ret.design_mapping.len() == 0)]
    pub(super) fn new() -> Self {
        Self {
            pattern_mapping: HashMap::new(),
            design_mapping: HashMap::new(),
        }
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    #[contracts::debug_ensures(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn insert(&mut self, pattern: CellRefWrapper<'p>, design: CellRefWrapper<'d>) {
        self.pattern_mapping.insert(pattern, design);
        self.design_mapping.insert(design, pattern);
    }

    // debug ensure that pattern & design mappings are consistent
    #[contracts::debug_ensures(self.debug_consistent(Some(pattern), ret))]
    pub(super) fn get_design_cell(
        &self,
        pattern: CellRefWrapper<'p>,
    ) -> Option<CellRefWrapper<'d>> {
        self.pattern_mapping.get(&pattern).map(|d| *d)
    }

    // debug ensure that pattern & design mappings are consistent
    #[contracts::debug_ensures(self.debug_consistent(ret, Some(design)))]
    pub(super) fn get_pattern_cell(
        &self,
        design: CellRefWrapper<'d>,
    ) -> Option<CellRefWrapper<'p>> {
        self.design_mapping.get(&design).map(|p| *p)
    }

    fn debug_consistent(
        &self,
        pattern: Option<CellRefWrapper<'p>>,
        design: Option<CellRefWrapper<'d>>,
    ) -> bool {
        if pattern.is_none() && design.is_none() {
            return true;
        }
        let pattern_in: CellRefWrapper<'p> = pattern.unwrap();
        let design_in: CellRefWrapper<'d> = design.unwrap();

        let design_result = self.pattern_mapping.get(&pattern_in).map(|&d| d);
        let pattern_result_cell = self.design_mapping.get(&design_in).map(|&p| p);
        match (design_result, pattern_result_cell) {
            (Some(d), Some(p)) => d == design_in && p == pattern_in,
            _ => false,
        }
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
    pub(super) fn design_mapping(&self) -> &HashMap<CellRefWrapper<'d>, CellRefWrapper<'p>> {
        &self.design_mapping
    }
    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn pattern_mapping(&self) -> &HashMap<CellRefWrapper<'p>, CellRefWrapper<'d>> {
        &self.pattern_mapping
    }
    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn sig(&self) -> Vec<usize> {
        let mut sig: Vec<usize> = self
            .pattern_mapping
            .values()
            .map(|d| d.debug_index())
            .collect();
        sig.sort_unstable();
        sig.dedup();
        sig
    }
}
