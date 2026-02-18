use std::borrow::Cow;
use std::fmt;

use prjunnamed_netlist::{Cell, CellRef, MetaItem, MetaItemRef, SourcePosition};

use crate::{CellKind, PhysicalCellId, SourceLine, SourceLocation, Wire};

// /// A wrapper around a netlist cell that provides stable identity and metadata access.
// #[derive(Clone)]
// pub struct CellWrapper<'a> {
//     /// The underlying netlist cell data.
//     cell: Cow<'a, Cell>,
//     /// Persistent identifier for this cell in the design.
//     debug_index: PhysicalCellId,
//     /// Reference to the cell's metadata if available.
//     metadata: MetaItemRef<'a>,
// }

// impl fmt::Debug for CellWrapper<'_> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         f.debug_struct("CellWrapper")
//             .field("cell", &self.get())
//             .field("metadata", &self.metadata)
//             .field("debug_index", &self.debug_index)
//             .finish()
//     }
// }

// impl PartialEq for CellWrapper<'_> {
//     fn eq(&self, other: &Self) -> bool {
//         self.debug_index == other.debug_index && self.cell == other.cell
//     }
// }

// impl Eq for CellWrapper<'_> {}

// impl Hash for CellWrapper<'_> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.debug_index.hash(state);
//     }
// }

// impl<'a> CellWrapper<'a> {
//     /// Returns the categorized type of the underlying cell.
//     #[must_use]
//     pub fn cell_type(&self) -> CellKind {
//         CellKind::from(self.cell.as_ref())
//     }

//     /// Returns a reference to the underlying netlist cell.
//     #[must_use]
//     pub fn get(&'a self) -> &'a Cell {
//         self.cell.as_ref()
//     }

//     /// Returns the unique debug index assigned by the netlist parser.
//     #[must_use]
//     pub const fn debug_index(&self) -> PhysicalCellId {
//         self.debug_index
//     }

//     /// Returns the metadata associated with this cell.
//     #[must_use]
//     pub const fn debug_info(&self) -> MetaItemRef<'a> {
//         self.metadata
//     }

//     /// Returns the name of the input port if this cell is an Input.
//     #[must_use]
//     pub const fn input_name(&self) -> Option<&str> {
//         match &self.cell {
//             Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
//             Cow::Owned(Cell::Input(name, _)) => Some(name.as_str()),
//             _ => None,
//         }
//     }

//     /// Returns the name of the output port if this cell is an Output.
//     #[must_use]
//     pub const fn output_name(&self) -> Option<&'a str> {
//         match &self.cell {
//             Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
//             _ => None,
//         }
//     }

//     /// Extracts source location information from cell metadata if available.
//     #[must_use]
//     pub fn get_source(&self) -> Option<SourceLocation> {
//         match self.metadata.get() {
//             MetaItem::Source { file, start, end } => {
//                 let lines = (start.line..=end.line)
//                     .map(|ln| self.calculate_line_span(ln, start, end))
//                     .collect();

//                 Some(SourceLocation {
//                     file: file.get(),
//                     lines,
//                 })
//             }
//             _ => None,
//         }
//     }

//     /// Calculates the column span for a specific line within a source range.
//     const fn calculate_line_span(
//         &self,
//         line_num: u32,
//         start: SourcePosition,
//         end: SourcePosition,
//     ) -> SourceLine {
//         let start_column = if line_num == start.line {
//             start.column as usize
//         } else {
//             1
//         };

//         let end_column = if line_num == end.line {
//             end.column as usize
//         } else {
//             0
//         };

//         SourceLine {
//             number: line_num as usize,
//             start_column,
//             end_column,
//         }
//     }

//     // /// Converts the wrapper and its specific metadata into a portable `CellInfo`.
//     // #[must_use]
//     // pub fn to_info(&self) -> CellInfo {
//     //     CellInfo {
//     //         id: self.debug_index,
//     //         kind: self.cell_type(),
//     //         source_loc: self.get_source(),
//     //     }
//     // }
// }

// impl<'a> From<CellRef<'a>> for CellWrapper<'a> {
//     fn from(val: CellRef<'a>) -> Self {
//         CellWrapper {
//             cell: val.get(),
//             debug_index: PhysicalCellId::new(val.debug_index() as u32),
//             metadata: val.metadata(),
//         }
//     }
// }

/// A wrapper around a netlist cell reference.
#[derive(Clone, Copy, PartialEq)]
pub struct CellWrapper<'a> {
    /// The underlying netlist cell reference.
    inner_cell_ref: CellRef<'a>,
}

impl fmt::Debug for CellWrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CellWrapper")
            .field("id", &self.debug_index())
            .field("kind", &self.cell_type())
            .finish()
    }
}

impl Eq for CellWrapper<'_> {}

impl<'a> CellWrapper<'a> {
    /// Creates a new wrapper from a cell reference.
    #[must_use]
    pub const fn new(inner: CellRef<'a>) -> Self {
        Self {
            inner_cell_ref: inner,
        }
    }

    pub(crate) const fn inner(&self) -> CellRef<'a> {
        self.inner_cell_ref
    }

    /// Returns the categorized type of the underlying cell.
    #[must_use]
    pub fn cell_type(&self) -> CellKind {
        CellKind::from(self.get().as_ref())
    }

    /// Returns a reference to the underlying netlist cell.
    #[must_use]
    pub fn get(&self) -> Cow<'a, Cell> {
        self.inner_cell_ref.get()
    }

    /// Returns the unique debug index assigned by the netlist parser.
    #[must_use]
    pub fn debug_index(&self) -> PhysicalCellId {
        PhysicalCellId::new(self.inner_cell_ref.debug_index() as u32)
    }

    /// Returns the metadata associated with this cell.
    #[must_use]
    pub fn metadata(&self) -> MetaItemRef<'a> {
        self.inner_cell_ref.metadata()
    }

    /// Returns the name of the input port if this cell is an Input.
    #[must_use]
    pub fn input_name(&self) -> Option<String> {
        match self.get().as_ref() {
            Cell::Input(name, _) => Some(name.clone()),
            _ => None,
        }
    }

    /// Returns the name of the output port if this cell is an Output.
    #[must_use]
    pub fn output_name(&self) -> Option<String> {
        match self.get().as_ref() {
            Cell::Output(name, _) => Some(name.clone()),
            _ => None,
        }
    }

    /// Extracts source location information from cell metadata.
    #[must_use]
    pub fn get_source(&self) -> Option<SourceLocation> {
        match self.metadata().get() {
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
        SourceLine {
            number: line_num as usize,
            start_column: if line_num == start.line {
                start.column as usize
            } else {
                1
            },
            end_column: if line_num == end.line {
                end.column as usize
            } else {
                0
            },
        }
    }

    pub fn input_wire(&self, port_name: &str) -> Option<Wire> {
        use prjunnamed_netlist::Cell;
        let cell = self.get();

        let val = match cell.as_ref() {
            Cell::And(a, b)
            | Cell::Or(a, b)
            | Cell::Xor(a, b)
            | Cell::Eq(a, b)
            | Cell::ULt(a, b)
            | Cell::SLt(a, b)
            | Cell::Mul(a, b) => match port_name {
                "a" => a,
                "b" => b,
                _ => return None,
            },
            Cell::Not(a) | Cell::Buf(a) => match port_name {
                "a" => a,
                _ => return None,
            },
            Cell::Mux(s, a, b) => match port_name {
                "sel" => return Some(Wire::single(*s)),
                "a" => a,
                "b" => b,
                _ => return None,
            },
            Cell::Dff(ff) => match port_name {
                "clk" => return Some(Wire::single(ff.clock.net())),
                "d" | "data_in" => &ff.data,
                "en" | "enable" => return Some(Wire::single(ff.enable.net())),
                "reset" | "rst" => return Some(Wire::single(ff.reset.net())),
                _ => return None,
            },
            _ => return None,
        };

        Some(Wire(val.clone()))
    }

    pub fn output_wire(&self) -> Wire {
        let width = match self.get().as_ref() {
            prjunnamed_netlist::Cell::Dff(ff) => ff.data.len(),
            _ => 1,
        };
        let base = self.debug_index().storage_key() as usize;
        (0..width)
            .map(|i| prjunnamed_netlist::Net::from_cell_index(base + i))
            .collect()
    }
}

impl<'a> From<CellRef<'a>> for CellWrapper<'a> {
    fn from(val: CellRef<'a>) -> Self {
        Self::new(val)
    }
}
