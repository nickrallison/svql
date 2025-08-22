use std::{borrow::Cow, hash::Hash};

use prjunnamed_netlist::{Cell, CellRef, Design, MetaItemRef, Net, Trit, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum CellKind {
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

impl CellKind {
    pub fn is_gate(&self) -> bool {
        matches!(
            self,
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
        )
    }
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellWrapper<'p> {
    pub cref: CellRef<'p>,
}

impl<'p> CellWrapper<'p> {
    pub fn new(cref: CellRef<'p>) -> Self {
        CellWrapper { cref }
    }
    pub fn cref(&self) -> CellRef<'p> {
        self.cref
    }
    pub fn debug_index(&self) -> usize {
        self.cref.debug_index()
    }
    pub fn get(self) -> Cow<'p, Cell> {
        self.cref.get()
    }

    pub fn metadata(&self) -> MetaItemRef<'p> {
        self.cref.metadata()
    }

    pub fn output_len(&self) -> usize {
        self.cref.output_len()
    }

    pub fn output(&self) -> Value {
        self.cref.output()
    }

    pub fn visit(&self, f: impl FnMut(Net)) {
        self.cref.visit(f)
    }

    pub fn replace(&self, to_cell: Cell) {
        self.cref.replace(to_cell)
    }

    pub fn append_metadata(&self, metadata: MetaItemRef<'p>) {
        self.cref.append_metadata(metadata)
    }

    pub fn unalive(&self) {
        self.cref.unalive()
    }

    pub fn design(self) -> &'p Design {
        self.cref.design()
    }
    pub fn maybe_name(&self) -> Option<&str> {
        match self.cref.get() {
            Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Debug for CellWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index: usize = self.cref.debug_index();
        let metadata: MetaItemRef = self.cref.metadata();

        f.debug_struct("CellWrapper")
            .field("index", &index)
            .field("meta", &metadata)
            .field("cell", self.cref.get().as_ref())
            .finish()
    }
}

impl<'a> From<CellRef<'a>> for CellWrapper<'a> {
    fn from(cref: CellRef<'a>) -> Self {
        CellWrapper { cref }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Source<'a> {
    Gate(CellWrapper<'a>, usize),
    Io(CellWrapper<'a>, usize),
    Const(Trit),
}

#[derive(Clone, Debug)]
pub(crate) struct CellPins<'a> {
    pub(crate) inputs: Vec<Source<'a>>,
}

pub(crate) fn is_gate_cell_ref(c: CellRef<'_>) -> bool {
    CellKind::from(c.get().as_ref()).is_gate()
}

pub(crate) fn get_input_cells<'a>(design: &'a Design) -> Vec<CellWrapper<'a>> {
    design
        .iter_cells()
        .filter(|cell_ref| matches!(cell_ref.get().as_ref(), Cell::Input(_, _)))
        .map(CellWrapper::from)
        .collect()
}

pub(crate) fn get_output_cells<'a>(design: &'a Design) -> Vec<CellWrapper<'a>> {
    design
        .iter_cells()
        .filter(|cell_ref| matches!(cell_ref.get().as_ref(), Cell::Output(_, _)))
        .map(CellWrapper::from)
        .collect()
}

pub(crate) fn input_name<'p>(cell: &CellWrapper<'p>) -> Option<&'p str> {
    match cell.cref().get() {
        std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
        _ => None,
    }
}

pub(crate) fn output_name<'p>(cell: &CellWrapper<'p>) -> Option<&'p str> {
    match cell.cref().get() {
        std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
        _ => None,
    }
}

pub(crate) fn extract_pins<'a>(cref: CellWrapper<'a>) -> CellPins<'a> {
    let mut inputs: Vec<Source<'a>> = Vec::new();
    cref.visit(|net| {
        inputs.push(net_to_source(cref.design(), net));
    });
    CellPins { inputs }
}

fn net_to_source<'a>(design: &'a Design, net: Net) -> Source<'a> {
    match design.find_cell(net) {
        Ok((src, bit)) => {
            if is_gate_cell_ref(src) {
                Source::Gate(src.into(), bit)
            } else {
                Source::Io(src.into(), bit)
            }
        }
        Err(trit) => Source::Const(trit),
    }
}

// #[cfg(test)]
// mod tests {
//     use prjunnamed_netlist::Design;

//     use crate::model::normalize::normalize_commutative;

//     use super::*;

//     lazy_static::lazy_static! {
//         static ref SDFFE: Design = crate::test_support::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
//     }

//     #[test]
//     fn test_is_gate_kind() {
//         // Gates
//         for k in [
//             CellKind::Buf,
//             CellKind::Not,
//             CellKind::And,
//             CellKind::Or,
//             CellKind::Xor,
//             CellKind::Mux,
//             CellKind::Adc,
//             CellKind::Aig,
//             CellKind::Eq,
//             CellKind::ULt,
//             CellKind::SLt,
//             CellKind::Shl,
//             CellKind::UShr,
//             CellKind::SShr,
//             CellKind::XShr,
//             CellKind::Mul,
//             CellKind::UDiv,
//             CellKind::UMod,
//             CellKind::SDivTrunc,
//             CellKind::SDivFloor,
//             CellKind::SModTrunc,
//             CellKind::SModFloor,
//             CellKind::Dff,
//         ] {
//             assert!(k.is_gate(), "kind {:?} must be considered a gate", k);
//         }

//         // Not gates
//         for k in [
//             CellKind::Input,
//             CellKind::Output,
//             CellKind::IoBuf,
//             CellKind::Assign,
//             CellKind::Match,
//             CellKind::Target,
//             CellKind::Other,
//             CellKind::Name,
//             CellKind::Debug,
//             CellKind::Memory,
//         ] {
//             assert!(!k.is_gate(), "kind {:?} must NOT be considered a gate", k);
//         }
//     }

//     #[test]
//     fn can_extract_some_pins() {
//         let d = &SDFFE;
//         for c in d.iter_cells() {
//             let _pins = extract_pins(c.into());
//         }
//     }

//     #[test]
//     fn commutative_sort_is_stable() {
//         let d = &SDFFE;
//         for c in d.iter_cells() {
//             let mut pins1 = extract_pins(c.into()).inputs;
//             let mut pins2 = extract_pins(c.into()).inputs;
//             normalize_commutative(&mut pins1);
//             normalize_commutative(&mut pins2);
//             assert_eq!(pins1.len(), pins2.len());
//             assert_eq!(pins1, pins2);
//         }
//     }
// }
