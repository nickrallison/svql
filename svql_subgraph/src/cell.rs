use std::borrow::Cow;
use std::fmt::Formatter;
use std::hash::Hash;

use prjunnamed_netlist::{Cell, CellRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CellType {
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

impl CellType {
    pub fn is_logic_gate(&self) -> bool {
        matches!(
            self,
            CellType::Buf
                | CellType::Not
                | CellType::And
                | CellType::Or
                | CellType::Xor
                | CellType::Mux
                | CellType::Adc
                | CellType::Aig
                | CellType::Eq
                | CellType::ULt
                | CellType::SLt
                | CellType::Shl
                | CellType::UShr
                | CellType::SShr
                | CellType::XShr
                | CellType::Mul
                | CellType::UDiv
                | CellType::UMod
                | CellType::SDivTrunc
                | CellType::SDivFloor
                | CellType::SModTrunc
                | CellType::SModFloor
                | CellType::Dff
        )
    }

    pub fn is_input(&self) -> bool {
        matches!(self, CellType::Input)
    }

    pub fn is_output(&self) -> bool {
        matches!(self, CellType::Output)
    }

    pub fn has_commutative_inputs(&self) -> bool {
        matches!(
            self,
            CellType::And | CellType::Or | CellType::Xor | CellType::Aig
        )
    }
}

impl From<&Cell> for CellType {
    fn from(cell: &Cell) -> Self {
        match cell {
            Cell::Buf(..) => CellType::Buf,
            Cell::Not(..) => CellType::Not,
            Cell::And(..) => CellType::And,
            Cell::Or(..) => CellType::Or,
            Cell::Xor(..) => CellType::Xor,
            Cell::Mux(..) => CellType::Mux,
            Cell::Adc(..) => CellType::Adc,
            Cell::Aig(..) => CellType::Aig,
            Cell::Eq(..) => CellType::Eq,
            Cell::ULt(..) => CellType::ULt,
            Cell::SLt(..) => CellType::SLt,
            Cell::Shl(..) => CellType::Shl,
            Cell::UShr(..) => CellType::UShr,
            Cell::SShr(..) => CellType::SShr,
            Cell::XShr(..) => CellType::XShr,
            Cell::Mul(..) => CellType::Mul,
            Cell::UDiv(..) => CellType::UDiv,
            Cell::UMod(..) => CellType::UMod,
            Cell::SDivTrunc(..) => CellType::SDivTrunc,
            Cell::SDivFloor(..) => CellType::SDivFloor,
            Cell::SModTrunc(..) => CellType::SModTrunc,
            Cell::SModFloor(..) => CellType::SModFloor,
            Cell::Match(..) => CellType::Match,
            Cell::Assign(..) => CellType::Assign,
            Cell::Dff(..) => CellType::Dff,
            Cell::Memory(..) => CellType::Memory,
            Cell::IoBuf(..) => CellType::IoBuf,
            Cell::Target(..) => CellType::Target,
            Cell::Other(..) => CellType::Other,
            Cell::Input(..) => CellType::Input,
            Cell::Output(..) => CellType::Output,
            Cell::Name(..) => CellType::Name,
            Cell::Debug(..) => CellType::Debug,
        }
    }
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CellWrapper<'a> {
    cell: Cow<'a, Cell>,
    cell_ref: CellRef<'a>,
}

impl<'a> CellWrapper<'a> {
    pub fn cell_type(&self) -> CellType {
        CellType::from(self.cell.as_ref())
    }
    pub fn get(&'a self) -> &'a Cell {
        self.cell.as_ref()
    }
    pub fn cell_ref(&'a self) -> CellRef<'a> {
        self.cell_ref
    }
    pub fn debug_index(&self) -> usize {
        self.cell_ref.debug_index()
    }

    pub fn input_name(&self) -> Option<&'a str> {
        match self.cell_ref.get() {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn output_name(&self) -> Option<&'a str> {
        match self.cell_ref.get() {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
}

impl<'a> Into<CellWrapper<'a>> for CellRef<'a> {
    fn into(self) -> CellWrapper<'a> {
        CellWrapper {
            cell: self.get(),
            cell_ref: self,
        }
    }
}
