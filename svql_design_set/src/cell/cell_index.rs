#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellIndex {
    idx: usize,
}

impl CellIndex {
    pub fn new(idx: usize) -> Self {
        CellIndex { idx }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.idx
    }
}
