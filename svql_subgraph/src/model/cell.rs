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
    pub fn is_input(&self) -> bool {
        matches!(self, CellKind::Input)
    }
    pub fn is_output(&self) -> bool {
        matches!(self, CellKind::Output)
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellRefWrapper<'a> {
    pub cref: CellRef<'a>,
}

impl<'p> CellRefWrapper<'p> {
    pub fn new(cref: CellRef<'p>) -> Self {
        CellRefWrapper { cref }
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

    pub fn input_name(&self) -> Option<&'p str> {
        match self.cref().get() {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
    pub fn output_name(&self) -> Option<&str> {
        match self.cref().get() {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
}

impl<'a> From<CellWrapper<'a>> for CellRefWrapper<'a> {
    fn from(wrapper: CellWrapper<'a>) -> Self {
        wrapper.cref
    }
}

impl std::fmt::Debug for CellRefWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index: usize = self.cref.debug_index();
        let metadata: MetaItemRef = self.cref.metadata();

        f.debug_struct("CellRefWrapper")
            .field("index", &index)
            .field("meta", &metadata)
            .field("cell", self.cref.get().as_ref())
            .finish()
    }
}

impl<'p> From<CellRef<'p>> for CellRefWrapper<'p> {
    fn from(cref: CellRef<'p>) -> Self {
        CellRefWrapper { cref }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CellWrapper<'p> {
    pub(crate) cref: CellRefWrapper<'p>,
    pub(crate) pins: Vec<Source<'p>>,
    pub(crate) kind: CellKind,
}

impl<'p> CellWrapper<'p> {
    pub(crate) fn new(cref: CellRef<'p>) -> Self {
        let mut pins: Vec<Source<'p>> = Vec::new();
        cref.visit(|net| {
            pins.push(net_to_source(cref.design(), net));
        });
        let kind = CellKind::from(cref.get().as_ref());
        let cref = cref.into();
        CellWrapper { cref, pins, kind }
    }
    pub(crate) fn cref(&self) -> CellRef<'p> {
        self.cref.cref
    }
    pub(crate) fn cref_wrapper(&self) -> CellRefWrapper<'p> {
        self.cref
    }
    pub(crate) fn debug_index(&self) -> usize {
        self.cref.cref.debug_index()
    }
    pub(crate) fn get(self) -> Cow<'p, Cell> {
        self.cref.cref.get()
    }

    pub(crate) fn metadata(&self) -> MetaItemRef<'p> {
        self.cref.cref.metadata()
    }

    pub(crate) fn output_len(&self) -> usize {
        self.cref.cref.output_len()
    }

    pub(crate) fn output(&self) -> Value {
        self.cref.cref.output()
    }

    pub(crate) fn visit(&self, f: impl FnMut(Net)) {
        self.cref.cref.visit(f)
    }

    pub(crate) fn replace(&self, to_cell: Cell) {
        self.cref.cref.replace(to_cell)
    }

    pub(crate) fn append_metadata(&self, metadata: MetaItemRef<'p>) {
        self.cref.cref.append_metadata(metadata)
    }

    pub(crate) fn unalive(&self) {
        self.cref.cref.unalive()
    }

    pub(crate) fn design(self) -> &'p Design {
        self.cref.cref.design()
    }

    pub(crate) fn input_name(&self) -> Option<&'p str> {
        match self.cref().get() {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
    pub(crate) fn output_name(&self) -> Option<&str> {
        match self.cref().get() {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
}

impl<'a> From<CellRef<'a>> for CellWrapper<'a> {
    fn from(cref: CellRef<'a>) -> Self {
        CellWrapper::new(cref)
    }
}

impl<'a> From<CellRefWrapper<'a>> for CellWrapper<'a> {
    fn from(wrapper: CellRefWrapper<'a>) -> Self {
        CellWrapper::new(wrapper.cref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Source<'a> {
    Gate(CellRefWrapper<'a>, usize),
    Io(CellRefWrapper<'a>, usize),
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
