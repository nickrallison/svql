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
        Self {
            pattern_mapping: HashMap::new(),
            design_mapping: HashMap::new(),
        }
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    #[contracts::debug_ensures(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn insert(&mut self, pattern: CellRef<'p>, design: CellRef<'d>) {
        self.pattern_mapping.insert(pattern, design);
        self.design_mapping.insert(design, pattern);
    }

    // debug ensure that pattern & design mappings are consistent
    #[contracts::debug_ensures(self.debug_consistent(Some(pattern), ret))]
    pub(super) fn get_design_cell(&self, pattern: CellRef<'p>) -> Option<CellRef<'d>> {
        self.pattern_mapping.get(&pattern).copied()
    }

    // debug ensure that pattern & design mappings are consistent
    #[contracts::debug_ensures(self.debug_consistent(ret, Some(design)))]
    pub(super) fn get_pattern_cell(&self, design: CellRef<'d>) -> Option<CellRef<'p>> {
        self.design_mapping.get(&design).copied()
    }

    fn debug_consistent(&self, pattern: Option<CellRef<'p>>, design: Option<CellRef<'d>>) -> bool {
        if pattern.is_none() && design.is_none() {
            return true;
        }
        let pattern_in: CellRef<'p> = pattern.unwrap();
        let design_in: CellRef<'d> = design.unwrap();

        let design_result = self.pattern_mapping.get(&pattern_in).copied();
        let pattern_result_cell = self.design_mapping.get(&design_in).copied();
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
    pub(super) fn design_mapping(&self) -> &HashMap<CellRef<'d>, CellRef<'p>> {
        &self.design_mapping
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn pattern_mapping(&self) -> &HashMap<CellRef<'p>, CellRef<'d>> {
        &self.pattern_mapping
    }

    #[contracts::debug_requires(self.pattern_mapping.len() == self.design_mapping.len())]
    pub(super) fn into_mapping(self) -> HashMap<CellRef<'p>, CellRef<'d>> {
        self.pattern_mapping
    }
}
