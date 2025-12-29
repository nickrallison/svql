//! Cell definitions and wrappers for subgraph matching.

use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::{borrow::Cow, fmt::Debug};

use prjunnamed_netlist::{Cell, CellRef, MetaItem, MetaItemRef, SourcePosition};

/// Categorizes netlist primitives into known types for matching.
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
    DLatchSr,
    ADLatch,
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
    /// Returns true if the cell represents a combinational or sequential logic gate.
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
                | CellKind::DLatchSr
                | CellKind::ADLatch
                | CellKind::Dff
        )
    }

    pub fn is_input(&self) -> bool {
        matches!(self, CellKind::Input)
    }

    pub fn is_output(&self) -> bool {
        matches!(self, CellKind::Output)
    }

    /// Returns true if the inputs to this cell can be swapped without changing logic.
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

/// A lightweight index wrapper for cells within a GraphIndex.
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

/// A wrapper around a netlist cell that provides stable identity and metadata access.
#[derive(Clone)]
pub struct CellWrapper<'a> {
    cell: Cow<'a, Cell>,
    debug_index: usize,
    metadata: MetaItemRef<'a>,
}

impl Debug for CellWrapper<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CellWrapper")
            .field("cell_type", &self.cell_type())
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl<'a> PartialEq for CellWrapper<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.debug_index == other.debug_index && self.cell == other.cell
    }
}

impl Eq for CellWrapper<'_> {}

impl<'a> Hash for CellWrapper<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cell.hash(state);
        self.debug_index.hash(state);
    }
}

impl<'a> CellWrapper<'a> {
    pub fn cell_type(&self) -> CellKind {
        CellKind::from(self.cell.as_ref())
    }

    pub fn get(&'a self) -> &'a Cell {
        self.cell.as_ref()
    }

    pub fn debug_index(&self) -> usize {
        self.debug_index
    }

    pub fn debug_info(&self) -> MetaItemRef<'a> {
        self.metadata
    }

    pub fn input_name(&self) -> Option<&str> {
        match &self.cell {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            std::borrow::Cow::Owned(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn output_name(&self) -> Option<&'a str> {
        match &self.cell {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Extracts source location information from cell metadata if available.
    pub fn get_source(&self) -> Option<SourceLocation> {
        match self.metadata.get() {
            MetaItem::Source { file, start, end } => {
                let lines = (start.line..=end.line)
                    .map(|ln| self.calculate_line_span(ln, start, end))
                    .collect();

                Some(SourceLocation {
                    file: file.get(),
                    lines,
                })
            }
            _ => None,
        }
    }

    fn calculate_line_span(
        &self,
        line_num: u32,
        start: SourcePosition,
        end: SourcePosition,
    ) -> SourceLine {
        let start_column = if line_num == start.line {
            start.column as usize
        } else {
            1
        };

        let end_column = if line_num == end.line {
            end.column as usize
        } else {
            0
        };

        SourceLine {
            number: line_num as usize,
            start_column,
            end_column,
        }
    }
}

impl<'a> From<CellRef<'a>> for CellWrapper<'a> {
    fn from(val: CellRef<'a>) -> Self {
        CellWrapper {
            cell: val.get(),
            debug_index: val.debug_index(),
            metadata: val.metadata(),
        }
    }
}

/// Represents a physical location in the source code.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: Arc<str>,
    pub lines: Vec<SourceLine>,
}

impl SourceLocation {
    /// Formats the source location for pretty-printed reports.
    pub fn report(&self) -> String {
        match self.lines.as_slice() {
            [] => format!("{}:<unknown>", self.file),
            [single] => format!("{}:{}", self.file, single.number),
            [first, .., last] => format!("{}:{}-{}", self.file, first.number, last.number),
        }
    }
}

/// Represents a specific line and column range within a source file.
#[derive(Debug, Clone)]
pub struct SourceLine {
    pub number: usize,
    pub start_column: usize,
    pub end_column: usize,
}

impl SourceLine {
    pub fn report(&self) -> String {
        if self.end_column == 0 {
            format!("Line {}, Col {}+", self.number, self.start_column)
        } else {
            format!(
                "Line {}, Col {}-{}",
                self.number, self.start_column, self.end_column
            )
        }
    }
}
