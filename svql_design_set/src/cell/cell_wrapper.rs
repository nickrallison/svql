use std::borrow::Cow;

use prjunnamed_netlist::{Cell, CellRef};

use crate::CellKind;

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
