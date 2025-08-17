use std::{borrow::Cow, collections::HashMap};

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

#[derive(Clone)]
pub struct InputCell<'p> {
    pub cref: CellRef<'p>,
}

impl<'p> InputCell<'p> {
    pub fn name(&self) -> Option<&'p str> {
        match self.cref.get() {
            Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn get_gates(&self) -> Vec<CellRef<'p>> {
        if matches!(self.cref.get().as_ref(), Cell::Input(_, _)) {
            let fanout = get_fanout(self.cref.design(), self.cref);
            fanout
        } else {
            vec![]
        }
    }
}

impl std::fmt::Debug for InputCell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InputCell")
            .field(&self.name().unwrap_or("<unnamed>"))
            .field(&self.cref.debug_index())
            .field(self.cref.get().as_ref())
            .finish()
    }
}

pub(crate) fn get_fanout<'a>(
    design: &'a Design,
    cell: CellRef<'a>,
) -> Vec<CellRef<'a>> {
    let mut fanout: Vec<CellRef<'a>> = Vec::new();

    for dest in design.iter_cells() {
        // Skip self to avoid self-loops in fanout
        if dest == cell {
            continue;
        }

        let mut driven_by_cell = false;
        dest.visit(|net| {
            if driven_by_cell {
                return; // already found a connection from `cell` to `dest`
            }
            if let Ok((src, _bit)) = design.find_cell(net) {
                if src == cell {
                    driven_by_cell = true;
                }
            }
        });

        if driven_by_cell {
            fanout.push(dest);
        }
    }

    fanout
}

// Helpers: return names and CellRefs, not cloned Cells
pub(crate) fn get_input_cells<'a>(design: &'a Design) -> Vec<InputCell<'a>> {
    design
        .iter_cells()
        .filter(|cell_ref| matches!(cell_ref.get().as_ref(), Cell::Input(_, _)))
        .map(|cref| InputCell { cref })
        .collect()
}

#[derive(Clone)]
pub struct OutputCell<'p> {
    pub cref: CellRef<'p>,
}

impl<'p> OutputCell<'p> {
    pub fn name(&self) -> Option<&'p str> {
        match self.cref.get() {
            Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn get_gate(&self) -> CellRef<'p> {
        let mut source: Option<CellRef<'p>> = None;
        if matches!(self.cref.get().as_ref(), Cell::Output(_, _)) {
            self.cref.visit(|net| {
                if let Ok((src, _bit)) = self.cref.design().find_cell(net) {
                    source = Some(src);
                }
            });
        }
        source.expect("Output cell should have a driving source")
    }
}

impl std::fmt::Debug for OutputCell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OutputCell")
            .field(&self.name().unwrap_or("<unnamed>"))
            .field(&self.cref.debug_index())
            .field(self.cref.get().as_ref())
            .finish()
    }
}

pub(crate) fn get_output_cells<'a>(design: &'a Design) -> Vec<OutputCell<'a>> {
    design
        .iter_cells()
        .filter(|cell_ref| matches!(cell_ref.get().as_ref(), Cell::Output(_, _)))
        .map(|cref| OutputCell { cref })
        .collect()
}

pub(crate) fn count_cells_by_kind(design: &Design) -> Vec<(CellKind, usize)> {
    let mut counts = HashMap::new();
    for cell in design.iter_cells() {
        let kind = cell_kind(&*cell.get());
        *counts.entry(kind).or_insert(0) += 1;
    }
    counts.into_iter().collect::<Vec<_>>()
}

pub(crate) fn cell_kind(c: &Cell) -> CellKind { CellKind::from(c) }

pub(crate) fn is_gate_kind(kind: CellKind) -> bool {
    matches!(
        kind,
        CellKind::Buf
            | CellKind::Not
            | CellKind::And
            | CellKind::Or
            | CellKind::Xor
            | CellKind::Mux
            | CellKind::Adc
            | CellKind::Aig
            | CellKind::Eq
            | CellKind::ULt
            | CellKind::SLt
            | CellKind::Shl
            | CellKind::UShr
            | CellKind::SShr
            | CellKind::XShr
            | CellKind::Mul
            | CellKind::UDiv
            | CellKind::UMod
            | CellKind::SDivTrunc
            | CellKind::SDivFloor
            | CellKind::SModTrunc
            | CellKind::SModFloor
            | CellKind::Dff
            // | CellKind::Input
            // | CellKind::Output
            // | CellKind::IoBuf
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_gate_kind() {
        // Gates
        for k in [
            CellKind::Buf, CellKind::Not, CellKind::And, CellKind::Or, CellKind::Xor,
            CellKind::Mux, CellKind::Adc, CellKind::Aig, CellKind::Eq, CellKind::ULt,
            CellKind::SLt, CellKind::Shl, CellKind::UShr, CellKind::SShr, CellKind::XShr,
            CellKind::Mul, CellKind::UDiv, CellKind::UMod, CellKind::SDivTrunc, CellKind::SDivFloor,
            CellKind::SModTrunc, CellKind::SModFloor, CellKind::Dff,
        ] {
            assert!(is_gate_kind(k), "kind {:?} must be considered a gate", k);
        }

        // Not gates
        for k in [
            CellKind::Input, CellKind::Output, CellKind::IoBuf, CellKind::Assign,
            CellKind::Match, CellKind::Target, CellKind::Other, CellKind::Name, CellKind::Debug,
            CellKind::Memory,
        ] {
            assert!(!is_gate_kind(k), "kind {:?} must NOT be considered a gate", k);
        }
    }
}