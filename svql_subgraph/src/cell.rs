use std::borrow::Cow;
use std::fmt::Formatter;
use std::hash::Hash;

use prjunnamed_netlist::{Cell, CellRef};

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
    DLatch,
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
    pub fn is_logic_gate(&self) -> bool {
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
                | CellKind::DLatch
                | CellKind::Dff
        )
    }

    pub fn is_input(&self) -> bool {
        matches!(self, CellKind::Input)
    }

    pub fn is_output(&self) -> bool {
        matches!(self, CellKind::Output)
    }

    pub fn has_commutative_inputs(&self) -> bool {
        matches!(
            self,
            CellKind::And | CellKind::Or | CellKind::Xor | CellKind::Aig
        )
    }
}

impl From<&Cell> for CellKind {
    fn from(cell: &Cell) -> Self {
        match cell {
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
            Cell::DLatch(..) => CellKind::DLatch,
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

impl std::fmt::Display for CellKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CellWrapper<'a> {
    cell: Cow<'a, Cell>,
    // cell_ref: CellRef<'a>,
    debug_index: usize,
}

impl<'a> CellWrapper<'a> {
    pub fn cell_type(&self) -> CellKind {
        CellKind::from(self.cell.as_ref())
    }
    pub fn get(&'a self) -> &'a Cell {
        self.cell.as_ref()
    }
    // pub fn cell_ref(&'a self) -> CellRef<'a> {
    //     self.cell_ref
    // }
    pub fn debug_index(&self) -> usize {
        self.debug_index
    }

    pub fn input_name(&self) -> Option<&'a str> {
        match &self.cell {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn output_name(&self) -> Option<&'a str> {
        match &self.cell {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
}

impl<'a> From<CellRef<'a>> for CellWrapper<'a> {
    fn from(val: CellRef<'a>) -> Self {
        CellWrapper {
            cell: val.get(),
            debug_index: val.debug_index(),
        }
    }
}
