use std::collections::{BTreeMap, HashMap};

use prjunnamed_netlist::{Cell, CellRef, Design};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CellKind {
    Buf,
    Not,
    And,
    Or,
    Xor,
    Mux,
    Adc,
    Aig,
    Eq,
    ULt,
    SLt,
    Shl,
    UShr,
    SShr,
    XShr,
    Mul,
    UDiv,
    UMod,
    SDivTrunc,
    SDivFloor,
    SModTrunc,
    SModFloor,
    Match,
    Assign,
    Dff,
    Memory,
    IoBuf,
    Target,
    Other,
    Input,
    Output,
    Name,
    Debug,
}

impl From<&Cell> for CellKind {
    fn from(c: &Cell) -> Self {
        match c {
            Cell::Buf(..) => CellKind::Buf,
            Cell::Not(..) => CellKind::Not,
            Cell::And(..) => CellKind::And,
            Cell::Or(..) => CellKind::Or,
            Cell::Xor(..) => CellKind::Xor,
            Cell::Mux(..) => CellKind::Mux,
            Cell::Adc(..) => CellKind::Adc,
            Cell::Aig(..) => CellKind::Aig,
            Cell::Eq(..) => CellKind::Eq,
            Cell::ULt(..) => CellKind::ULt,
            Cell::SLt(..) => CellKind::SLt,
            Cell::Shl(..) => CellKind::Shl,
            Cell::UShr(..) => CellKind::UShr,
            Cell::SShr(..) => CellKind::SShr,
            Cell::XShr(..) => CellKind::XShr,
            Cell::Mul(..) => CellKind::Mul,
            Cell::UDiv(..) => CellKind::UDiv,
            Cell::UMod(..) => CellKind::UMod,
            Cell::SDivTrunc(..) => CellKind::SDivTrunc,
            Cell::SDivFloor(..) => CellKind::SDivFloor,
            Cell::SModTrunc(..) => CellKind::SModTrunc,
            Cell::SModFloor(..) => CellKind::SModFloor,
            Cell::Match(..) => CellKind::Match,
            Cell::Assign(..) => CellKind::Assign,
            Cell::Dff(..) => CellKind::Dff,
            Cell::Memory(..) => CellKind::Memory,
            Cell::IoBuf(..) => CellKind::IoBuf,
            Cell::Target(..) => CellKind::Target,
            Cell::Other(..) => CellKind::Other,
            Cell::Input(..) => CellKind::Input,
            Cell::Output(..) => CellKind::Output,
            Cell::Name(..) => CellKind::Name,
            Cell::Debug(..) => CellKind::Debug,
        }
    }
}

// Define what we consider a "real gate" in this context: skip IO, metadata, targets, memories, etc.
fn is_gate_kind(kind: CellKind) -> bool {
    !matches!(
        kind,
        CellKind::Input
            | CellKind::Output
            | CellKind::Name
    )
}

pub struct CellTypeIndex<'a> {
    design: &'a Design,
    by_kind: HashMap<CellKind, Vec<CellRef<'a>>>,
    // Optional: map counts to kinds for quick min lookup
    by_count: BTreeMap<usize, Vec<CellKind>>, // count -> kinds
}

impl<'a> CellTypeIndex<'a> {
    pub fn build(design: &'a Design) -> Self {
        let mut by_kind: HashMap<CellKind, Vec<CellRef<'a>>> = HashMap::new();
        for cell in design.iter_cells() {
            let kind = CellKind::from(&*cell.get());
            by_kind.entry(kind).or_default().push(cell);
        }
        let mut by_count: BTreeMap<usize, Vec<CellKind>> = BTreeMap::new();
        for (kind, vec) in by_kind.iter() {
            by_count.entry(vec.len()).or_default().push(*kind);
        }
        Self { design, by_kind, by_count }
    }

    // Original: all kinds
    pub fn least_common_type(&self) -> Option<CellKind> {
        self.by_count.first_key_value().and_then(|(_cnt, kinds)| kinds.first().copied())
    }

    // New: least common among gate kinds only
    pub fn least_common_gate_type(&self) -> Option<CellKind> {
        for (_cnt, kinds) in self.by_count.iter() {
            if let Some(k) = kinds.iter().find(|k| is_gate_kind(**k)).copied() {
                return Some(k);
            }
        }
        None
    }

    // Return an iterator over all cells of a specific kind
    pub fn iter_cells_of(&self, kind: CellKind) -> impl Iterator<Item = CellRef<'a>> + '_ {
        self.by_kind.get(&kind).into_iter().flat_map(|v| v.iter().copied())
    }

    // Iterator-over-iterators: buckets of gate cells, rarest first.
    // Each item is a boxed iterator over CellRef<'a> to keep the outer iterator object-safe.
    pub fn iter_gate_buckets_rarest_first(
        &'a self,
    ) -> impl Iterator<Item = Box<dyn Iterator<Item = CellRef<'a>> + 'a>> + 'a {
        let kinds_in_order: Vec<CellKind> = self
            .by_count
            .iter()
            .flat_map(|(_cnt, kinds)| kinds.iter().filter(|k| is_gate_kind(**k)).copied())
            .collect();
        kinds_in_order
            .into_iter()
            .map(move |k| Box::new(self.iter_cells_of(k)) as Box<dyn Iterator<Item = CellRef<'a>> + 'a>)
    }

    // Same as above, but include the kind label with each bucket.
    pub fn iter_gate_kind_buckets_rarest_first(
        &'a self,
    ) -> impl Iterator<Item = (CellKind, Box<dyn Iterator<Item = CellRef<'a>> + 'a>)> + 'a {
        let kinds_in_order: Vec<CellKind> = self
            .by_count
            .iter()
            .flat_map(|(_cnt, kinds)| kinds.iter().filter(|k| is_gate_kind(**k)).copied())
            .collect();
        kinds_in_order
            .into_iter()
            .map(move |k| (k, Box::new(self.iter_cells_of(k)) as Box<dyn Iterator<Item = CellRef<'a>> + 'a>))
    }

    // Convenience: get an iterator over the least-common type (all kinds)
    pub fn least_common_cells(&self) -> Option<impl Iterator<Item = CellRef<'a>> + '_> {
        self.least_common_type().map(|k| self.iter_cells_of(k))
    }

    pub fn design(&self) -> &'a Design { self.design }
}

// Simple helpers for one-off queries (O(N) build each call). Prefer CellTypeIndex for repeated use.
pub fn least_common_cell_kind(design: &Design) -> Option<CellKind> {
    CellTypeIndex::build(design).least_common_type()
}

pub fn cells_of_kind<'a>(design: &'a Design, kind: CellKind) -> Vec<CellRef<'a>> {
    design
        .iter_cells()
        .filter(|c| CellKind::from(&*c.get()) == kind)
        .collect()
}
