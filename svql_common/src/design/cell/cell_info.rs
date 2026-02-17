// use crate::{CellKind, PhysicalCellId, SourceLocation};

// /// High-level metadata about a cell extracted from the netlist.
// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
// pub struct CellInfo {
//     /// Unique identifier to allow equality checks across design loads.
//     pub id: PhysicalCellId,
//     /// The primitive type of the cell.
//     pub kind: CellKind,
//     /// File and line information if available in the netlist.
//     pub source_loc: Option<SourceLocation>,
// }

// impl CellInfo {
//     /// Retrieves the source code location of the cell if available.
//     #[must_use]
//     pub fn get_source(&self) -> Option<SourceLocation> {
//         self.source_loc.clone()
//     }
// }
