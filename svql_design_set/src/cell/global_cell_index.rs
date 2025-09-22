#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlobalCellIndex<'a> {
    idx: usize,
    module_key: &'a str,
}

impl<'a> GlobalCellIndex<'a> {
    pub fn new(idx: usize, module_key: &'a str) -> Self {
        GlobalCellIndex { idx, module_key }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.idx
    }

    #[inline]
    pub fn module_key(&self) -> &'a str {
        self.module_key
    }
}
