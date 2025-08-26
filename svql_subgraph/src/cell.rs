use std::fmt::{Display, Formatter};
use std::hash::Hash;

use prjunnamed_netlist::{Cell, CellRef, Design, Net, Trit};

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
    pub fn is_input(&self) -> bool {
        matches!(self, CellKind::Input)
    }
    pub fn is_output(&self) -> bool {
        matches!(self, CellKind::Output)
    }
    pub fn is_commutative_inputs(&self) -> bool {
        matches!(
            self,
            CellKind::And | CellKind::Or | CellKind::Xor | CellKind::Aig
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
impl std::fmt::Display for CellKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellWrapper<'p> {
    cref: CellRef<'p>,
    pins: Vec<Source<'p>>,
    kind: CellKind,
}

impl<'p> CellWrapper<'p> {
    pub fn new(cref: CellRef<'p>) -> Self {
        let mut pins: Vec<Source<'p>> = Vec::new();
        cref.visit(|net| {
            pins.push(net_to_source(cref.design(), net));
        });
        let kind = CellKind::from(cref.get().as_ref());
        let cref = cref.into();
        CellWrapper { cref, pins, kind }
    }
    pub fn cref(&self) -> CellRef<'p> {
        self.cref
    }

    pub fn debug_index(&self) -> usize {
        self.cref.debug_index()
    }

    pub fn summary(&self) -> String {
        let iname = self.input_name().unwrap_or("");
        let oname = self.output_name().unwrap_or("");
        let n = if !iname.is_empty() { iname } else { oname };
        if n.is_empty() {
            format!("#{} {:?}", self.cref.debug_index(), self.kind)
        } else {
            format!("#{} {:?}({})", self.cref.debug_index(), self.kind, n)
        }
    }

    pub fn input_name(&self) -> Option<&'p str> {
        match self.cref().get() {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
    pub fn output_name(&self) -> Option<&'p str> {
        match self.cref().get() {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn kind(&self) -> CellKind {
        self.kind
    }
    pub fn pins(&self) -> &[Source<'p>] {
        &self.pins
    }
}

impl<'a> From<CellRef<'a>> for CellWrapper<'a> {
    fn from(cref: CellRef<'a>) -> Self {
        CellWrapper::new(cref)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source<'a> {
    Gate(CellRef<'a>, usize),
    Io(CellRef<'a>, usize),
    Const(Trit),
}

fn net_to_source<'a>(design: &'a Design, net: Net) -> Source<'a> {
    match design.find_cell(net) {
        Ok((src, bit)) => {
            if CellKind::from(src.get().as_ref()).is_gate() {
                Source::Gate(src.into(), bit)
            } else {
                Source::Io(src.into(), bit)
            }
        }
        Err(trit) => Source::Const(trit),
    }
}
