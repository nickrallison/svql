use std::fmt::Formatter;
use std::hash::Hash;

use prjunnamed_netlist::{Cell, CellRef, Design, Net, Trit};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeType {
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

impl NodeType {
    pub fn is_logic_gate(&self) -> bool {
        matches!(
            self,
            NodeType::Buf
                | NodeType::Not
                | NodeType::And
                | NodeType::Or
                | NodeType::Xor
                | NodeType::Mux
                | NodeType::Adc
                | NodeType::Aig
                | NodeType::Eq
                | NodeType::ULt
                | NodeType::SLt
                | NodeType::Shl
                | NodeType::UShr
                | NodeType::SShr
                | NodeType::XShr
                | NodeType::Mul
                | NodeType::UDiv
                | NodeType::UMod
                | NodeType::SDivTrunc
                | NodeType::SDivFloor
                | NodeType::SModTrunc
                | NodeType::SModFloor
                | NodeType::Dff
        )
    }

    pub fn is_input(&self) -> bool {
        matches!(self, NodeType::Input)
    }

    pub fn is_output(&self) -> bool {
        matches!(self, NodeType::Output)
    }

    pub fn has_commutative_inputs(&self) -> bool {
        matches!(
            self,
            NodeType::And | NodeType::Or | NodeType::Xor | NodeType::Aig
        )
    }
}

impl From<&Cell> for NodeType {
    fn from(cell: &Cell) -> Self {
        match cell {
            Cell::Buf(..) => NodeType::Buf,
            Cell::Not(..) => NodeType::Not,
            Cell::And(..) => NodeType::And,
            Cell::Or(..) => NodeType::Or,
            Cell::Xor(..) => NodeType::Xor,
            Cell::Mux(..) => NodeType::Mux,
            Cell::Adc(..) => NodeType::Adc,
            Cell::Aig(..) => NodeType::Aig,
            Cell::Eq(..) => NodeType::Eq,
            Cell::ULt(..) => NodeType::ULt,
            Cell::SLt(..) => NodeType::SLt,
            Cell::Shl(..) => NodeType::Shl,
            Cell::UShr(..) => NodeType::UShr,
            Cell::SShr(..) => NodeType::SShr,
            Cell::XShr(..) => NodeType::XShr,
            Cell::Mul(..) => NodeType::Mul,
            Cell::UDiv(..) => NodeType::UDiv,
            Cell::UMod(..) => NodeType::UMod,
            Cell::SDivTrunc(..) => NodeType::SDivTrunc,
            Cell::SDivFloor(..) => NodeType::SDivFloor,
            Cell::SModTrunc(..) => NodeType::SModTrunc,
            Cell::SModFloor(..) => NodeType::SModFloor,
            Cell::Match(..) => NodeType::Match,
            Cell::Assign(..) => NodeType::Assign,
            Cell::Dff(..) => NodeType::Dff,
            Cell::Memory(..) => NodeType::Memory,
            Cell::IoBuf(..) => NodeType::IoBuf,
            Cell::Target(..) => NodeType::Target,
            Cell::Other(..) => NodeType::Other,
            Cell::Input(..) => NodeType::Input,
            Cell::Output(..) => NodeType::Output,
            Cell::Name(..) => NodeType::Name,
            Cell::Debug(..) => NodeType::Debug,
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeSource<'a> {
    Gate(CellRef<'a>, usize),
    Io(CellRef<'a>, usize),
    Const(Trit),
}

pub fn net_to_source<'a>(design: &'a Design, net: Net) -> NodeSource<'a> {
    match design.find_cell(net) {
        Ok((src, bit)) => {
            if NodeType::from(src.get().as_ref()).is_logic_gate() {
                NodeSource::Gate(src, bit)
            } else {
                NodeSource::Io(src, bit)
            }
        }
        Err(trit) => NodeSource::Const(trit),
    }
}
