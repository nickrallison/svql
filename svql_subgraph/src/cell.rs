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
    #[must_use] 
    pub const fn is_logic_gate(&self) -> bool {
        matches!(
            self,
            Self::Buf
                | Self::Not
                | Self::And
                | Self::Or
                | Self::Xor
                | Self::Mux
                | Self::Adc
                | Self::Aig
                | Self::Eq
                | Self::ULt
                | Self::SLt
                | Self::Shl
                | Self::UShr
                | Self::SShr
                | Self::XShr
                | Self::Mul
                | Self::UDiv
                | Self::UMod
                | Self::SDivTrunc
                | Self::SDivFloor
                | Self::SModTrunc
                | Self::SModFloor
                | Self::DLatchSr
                | Self::ADLatch
                | Self::Dff
        )
    }

    /// Returns true if the cell is an input port.
    #[must_use] 
    pub const fn is_input(&self) -> bool {
        matches!(self, Self::Input)
    }

    /// Returns true if the cell is an output port.
    #[must_use] 
    pub const fn is_output(&self) -> bool {
        matches!(self, Self::Output)
    }

    /// Returns true if the inputs to this cell can be swapped without changing logic.
    #[must_use] 
    pub const fn has_commutative_inputs(&self) -> bool {
        matches!(
            self,
            Self::And | Self::Or | Self::Xor | Self::Aig
        )
    }
}

impl From<&Cell> for CellKind {
    fn from(cell: &Cell) -> Self {
        match cell {
            Cell::Buf(..) => Self::Buf,
            Cell::Not(..) => Self::Not,
            Cell::And(..) => Self::And,
            Cell::Or(..) => Self::Or,
            Cell::Xor(..) => Self::Xor,
            Cell::Mux(..) => Self::Mux,
            Cell::Adc(..) => Self::Adc,
            Cell::Aig(..) => Self::Aig,
            Cell::Eq(..) => Self::Eq,
            Cell::ULt(..) => Self::ULt,
            Cell::SLt(..) => Self::SLt,
            Cell::Shl(..) => Self::Shl,
            Cell::UShr(..) => Self::UShr,
            Cell::SShr(..) => Self::SShr,
            Cell::XShr(..) => Self::XShr,
            Cell::Mul(..) => Self::Mul,
            Cell::UDiv(..) => Self::UDiv,
            Cell::UMod(..) => Self::UMod,
            Cell::SDivTrunc(..) => Self::SDivTrunc,
            Cell::SDivFloor(..) => Self::SDivFloor,
            Cell::SModTrunc(..) => Self::SModTrunc,
            Cell::SModFloor(..) => Self::SModFloor,
            Cell::Match(..) => Self::Match,
            Cell::Assign(..) => Self::Assign,
            Cell::Dff(..) => Self::Dff,
            Cell::Memory(..) => Self::Memory,
            Cell::IoBuf(..) => Self::IoBuf,
            Cell::Target(..) => Self::Target,
            Cell::Other(..) => Self::Other,
            Cell::Input(..) => Self::Input,
            Cell::Output(..) => Self::Output,
            Cell::Name(..) => Self::Name,
            Cell::Debug(..) => Self::Debug,
        }
    }
}

impl std::fmt::Display for CellKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A lightweight index wrapper for cells within a `GraphIndex`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellIndex {
    idx: usize,
}

impl CellIndex {
    /// Creates a new `CellIndex` from a raw usize.
    #[must_use] 
    pub const fn new(idx: usize) -> Self {
        Self { idx }
    }

    /// Returns the raw index value.
    #[inline]
    #[must_use] 
    pub const fn index(&self) -> usize {
        self.idx
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CellInfo {
    /// Unique identifier (`debug_index`) to allow equality checks
    pub id: usize,
    /// The type of cell (AND, OR, DFF, etc.)
    pub kind: CellKind,
    /// Source code location (`Arc<str>` is cheap to clone and owned)
    pub source_loc: Option<SourceLocation>,
}

impl CellInfo {
    /// Retrieves the source code location of the cell if available.
    #[must_use] 
    pub fn get_source(&self) -> Option<SourceLocation> {
        self.source_loc.clone()
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
            .field("cell", &self.get())
            .field("metadata", &self.metadata)
            .field("debug_index", &self.debug_index)
            .finish()
    }
}

impl PartialEq for CellWrapper<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.debug_index == other.debug_index && self.cell == other.cell
    }
}

impl Eq for CellWrapper<'_> {}

impl Hash for CellWrapper<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cell.hash(state);
        self.debug_index.hash(state);
    }
}

impl<'a> CellWrapper<'a> {
    /// Returns the categorized type of the underlying cell.
    #[must_use] 
    pub fn cell_type(&self) -> CellKind {
        CellKind::from(self.cell.as_ref())
    }

    /// Returns a reference to the underlying netlist cell.
    #[must_use] 
    pub fn get(&'a self) -> &'a Cell {
        self.cell.as_ref()
    }

    /// Returns the unique debug index assigned by the netlist parser.
    #[must_use] 
    pub const fn debug_index(&self) -> usize {
        self.debug_index
    }

    /// Returns the metadata associated with this cell.
    #[must_use] 
    pub const fn debug_info(&self) -> MetaItemRef<'a> {
        self.metadata
    }

    /// Returns the name of the input port if this cell is an Input.
    #[must_use] 
    pub const fn input_name(&self) -> Option<&str> {
        match &self.cell {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            std::borrow::Cow::Owned(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Returns the name of the output port if this cell is an Output.
    #[must_use] 
    pub const fn output_name(&self) -> Option<&'a str> {
        match &self.cell {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    /// Extracts source location information from cell metadata if available.
    #[must_use] 
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

    /// Calculates the column span for a specific line within a source range.
    const fn calculate_line_span(
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

    #[must_use] 
    pub fn to_info(&self) -> CellInfo {
        CellInfo {
            id: self.debug_index,
            kind: self.cell_type(),
            source_loc: self.get_source(),
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub file: Arc<str>,
    pub lines: Vec<SourceLine>,
}

impl SourceLocation {
    /// Formats the source location for pretty-printed reports.
    #[must_use] 
    pub fn report(&self) -> String {
        match self.lines.as_slice() {
            [] => format!("{}:<unknown>", self.file),
            [single] => format!("{}:{}", self.file, single.number),
            [first, .., last] => format!("{}:{}-{}", self.file, first.number, last.number),
        }
    }
}

/// Represents a specific line and column range within a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLine {
    pub number: usize,
    pub start_column: usize,
    pub end_column: usize,
}

impl SourceLine {
    /// Formats the line and column range for reporting.
    #[must_use] 
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
