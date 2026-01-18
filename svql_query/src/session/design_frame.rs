//! Design data stored as polars DataFrames.
//!
//! Replaces the traditional `GraphIndex` lookup tables with columnar storage.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use polars::prelude::*;
use prjunnamed_netlist::{Cell, Design};

use super::SessionError;
use crate::subgraph::cell::{CellKind, SourceLine, SourceLocation};

/// A row from the cells DataFrame.
#[derive(Debug, Clone)]
pub struct CellRow {
    pub cell_id: u32,
    pub kind: CellKind,
    pub source_file: Option<Arc<str>>,
    pub source_lines: Vec<SourceLine>,
}

impl CellRow {
    /// Converts to CellInfo for compatibility with existing code.
    pub fn to_cell_info(&self) -> crate::subgraph::cell::CellInfo {
        crate::subgraph::cell::CellInfo {
            id: self.cell_id as usize,
            kind: self.kind,
            source_loc: self.source_location(),
        }
    }

    /// Returns the source location if available.
    pub fn source_location(&self) -> Option<SourceLocation> {
        self.source_file.as_ref().map(|file| SourceLocation {
            file: file.clone(),
            lines: self.source_lines.clone(),
        })
    }
}

/// Design data stored in DataFrame format.
///
/// Schema:
/// - `cells`: DataFrame with columns [cell_id, kind, source_file, source_start_line, source_end_line]
/// - `connectivity`: DataFrame with columns [from_cell_id, to_cell_id, port_idx]
/// - `io_ports`: DataFrame with columns [port_name, cell_id, is_input]
#[derive(Debug, Clone)]
pub struct DesignFrame {
    /// Cell information: [cell_id (u32), kind (u8), source_file (str), source_start (u32), source_end (u32)]
    cells: DataFrame,
    /// Connectivity edges: [from_id (u32), to_id (u32), port_idx (u32)]
    connectivity: DataFrame,
    /// I/O port mappings: [name (str), cell_id (u32), is_input (bool)]
    io_ports: DataFrame,
    /// Fast lookup: cell_id -> row index in cells DataFrame
    cell_id_to_row: HashMap<u32, usize>,
    /// Fast lookup: (from_id) -> set of to_ids (fanout)
    fanout_index: HashMap<u32, HashSet<u32>>,
    /// Fast lookup: (to_id) -> set of from_ids (fanin)
    fanin_index: HashMap<u32, HashSet<u32>>,
    /// Fast lookup: cell_kind -> list of cell_ids
    kind_index: HashMap<CellKind, Vec<u32>>,
}

impl DesignFrame {
    /// Builds a DesignFrame from a prjunnamed Design.
    pub fn from_design(design: &Design) -> Result<Self, SessionError> {
        let start = std::time::Instant::now();

        // Collect cell data
        let mut cell_ids: Vec<u32> = Vec::new();
        let mut cell_kinds: Vec<u8> = Vec::new();
        let mut source_files: Vec<Option<String>> = Vec::new();
        let mut source_starts: Vec<Option<u32>> = Vec::new();
        let mut source_ends: Vec<Option<u32>> = Vec::new();

        // Collect connectivity data
        let mut from_ids: Vec<u32> = Vec::new();
        let mut to_ids: Vec<u32> = Vec::new();
        let mut port_indices: Vec<u32> = Vec::new();

        // Collect I/O port data
        let mut port_names: Vec<String> = Vec::new();
        let mut port_cell_ids: Vec<u32> = Vec::new();
        let mut port_is_input: Vec<bool> = Vec::new();

        // Build indices
        let mut cell_id_to_row: HashMap<u32, usize> = HashMap::new();
        let mut fanout_index: HashMap<u32, HashSet<u32>> = HashMap::new();
        let mut fanin_index: HashMap<u32, HashSet<u32>> = HashMap::new();
        let mut kind_index: HashMap<CellKind, Vec<u32>> = HashMap::new();

        // Iterate cells in topological order
        for cell_ref in design.iter_cells_topo().rev() {
            let cell = cell_ref.get();
            let kind = CellKind::from(cell.as_ref());

            // Skip Name cells (metadata only)
            if matches!(kind, CellKind::Name) {
                continue;
            }

            let cell_id = cell_ref.debug_index() as u32;
            let row_idx = cell_ids.len();

            // Store cell data
            cell_ids.push(cell_id);
            cell_kinds.push(kind as u8);

            // Extract source location from metadata
            let meta = cell_ref.metadata();
            if let prjunnamed_netlist::MetaItem::Source { file, start, end } = meta.get() {
                source_files.push(Some(file.get().to_string()));
                source_starts.push(Some(start.line));
                source_ends.push(Some(end.line));
            } else {
                source_files.push(None);
                source_starts.push(None);
                source_ends.push(None);
            }

            // Build indices
            cell_id_to_row.insert(cell_id, row_idx);
            kind_index.entry(kind).or_default().push(cell_id);

            // Handle I/O ports
            match cell.as_ref() {
                Cell::Input(name, _) => {
                    port_names.push(name.to_string());
                    port_cell_ids.push(cell_id);
                    port_is_input.push(true);
                }
                Cell::Output(name, _) => {
                    port_names.push(name.to_string());
                    port_cell_ids.push(cell_id);
                    port_is_input.push(false);
                }
                _ => {}
            }

            // Build connectivity by visiting cell inputs
            cell_ref.visit(|net| {
                if let Ok((source_ref, source_pin_idx)) = design.find_cell(net) {
                    let source_id = source_ref.debug_index() as u32;

                    from_ids.push(source_id);
                    to_ids.push(cell_id);
                    port_indices.push(source_pin_idx as u32);

                    fanout_index.entry(source_id).or_default().insert(cell_id);
                    fanin_index.entry(cell_id).or_default().insert(source_id);
                }
            });
        }

        // Build DataFrames
        let cells = df! {
            "cell_id" => &cell_ids,
            "kind" => &cell_kinds,
            "source_file" => &source_files,
            "source_start" => &source_starts,
            "source_end" => &source_ends,
        }?;

        let connectivity = df! {
            "from_id" => &from_ids,
            "to_id" => &to_ids,
            "port_idx" => &port_indices,
        }?;

        let io_ports = df! {
            "name" => &port_names,
            "cell_id" => &port_cell_ids,
            "is_input" => &port_is_input,
        }?;

        tracing::debug!(
            "DesignFrame built in {:?} for {} cells, {} edges",
            start.elapsed(),
            cell_ids.len(),
            from_ids.len()
        );

        Ok(Self {
            cells,
            connectivity,
            io_ports,
            cell_id_to_row,
            fanout_index,
            fanin_index,
            kind_index,
        })
    }

    /// Returns the number of cells.
    pub fn num_cells(&self) -> usize {
        self.cells.height()
    }

    /// Returns the number of connectivity edges.
    pub fn num_edges(&self) -> usize {
        self.connectivity.height()
    }

    /// Returns the cells DataFrame.
    pub fn cells_df(&self) -> &DataFrame {
        &self.cells
    }

    /// Returns the connectivity DataFrame.
    pub fn connectivity_df(&self) -> &DataFrame {
        &self.connectivity
    }

    /// Returns the I/O ports DataFrame.
    pub fn io_ports_df(&self) -> &DataFrame {
        &self.io_ports
    }

    /// Gets a cell by its ID.
    pub fn get_cell(&self, cell_id: u32) -> Option<CellRow> {
        let row_idx = *self.cell_id_to_row.get(&cell_id)?;
        self.get_cell_by_row(row_idx)
    }

    /// Gets a cell by its row index in the DataFrame.
    pub fn get_cell_by_row(&self, row_idx: usize) -> Option<CellRow> {
        if row_idx >= self.cells.height() {
            return None;
        }

        let cell_id = self
            .cells
            .column("cell_id")
            .ok()?
            .u32()
            .ok()?
            .get(row_idx)? as u32;

        let kind_u8 = self.cells.column("kind").ok()?.u8().ok()?.get(row_idx)?;
        let kind = cell_kind_from_u8(kind_u8);

        let source_file = self
            .cells
            .column("source_file")
            .ok()?
            .str()
            .ok()?
            .get(row_idx)
            .map(|s| Arc::from(s));

        let source_start = self
            .cells
            .column("source_start")
            .ok()?
            .u32()
            .ok()?
            .get(row_idx);
        let source_end = self
            .cells
            .column("source_end")
            .ok()?
            .u32()
            .ok()?
            .get(row_idx);

        let source_lines = match (source_start, source_end) {
            (Some(start), Some(end)) => (start..=end)
                .map(|ln| SourceLine {
                    number: ln as usize,
                    start_column: if ln == start { 1 } else { 1 },
                    end_column: 0,
                })
                .collect(),
            _ => vec![],
        };

        Some(CellRow {
            cell_id,
            kind,
            source_file,
            source_lines,
        })
    }

    /// Checks if two cells are connected (from -> to).
    pub fn is_connected(&self, from_id: u32, to_id: u32) -> bool {
        self.fanout_index
            .get(&from_id)
            .map(|fanout| fanout.contains(&to_id))
            .unwrap_or(false)
    }

    /// Returns the fanout cell IDs for a given cell.
    pub fn fanout(&self, cell_id: u32) -> Option<&HashSet<u32>> {
        self.fanout_index.get(&cell_id)
    }

    /// Returns the fanin cell IDs for a given cell.
    pub fn fanin(&self, cell_id: u32) -> Option<&HashSet<u32>> {
        self.fanin_index.get(&cell_id)
    }

    /// Returns all cell IDs of a given kind.
    pub fn cells_of_kind(&self, kind: CellKind) -> Option<&[u32]> {
        self.kind_index.get(&kind).map(|v| v.as_slice())
    }

    /// Returns input port names and their cell IDs.
    pub fn input_ports(&self) -> impl Iterator<Item = (String, u32)> + '_ {
        let names = self.io_ports.column("name").unwrap().str().unwrap();
        let cell_ids = self.io_ports.column("cell_id").unwrap().u32().unwrap();
        let is_input = self.io_ports.column("is_input").unwrap().bool().unwrap();

        (0..self.io_ports.height()).filter_map(move |i| {
            if is_input.get(i)? {
                Some((names.get(i)?.to_string(), cell_ids.get(i)?))
            } else {
                None
            }
        })
    }

    /// Returns output port names and their cell IDs.
    pub fn output_ports(&self) -> impl Iterator<Item = (String, u32)> + '_ {
        let names = self.io_ports.column("name").unwrap().str().unwrap();
        let cell_ids = self.io_ports.column("cell_id").unwrap().u32().unwrap();
        let is_input = self.io_ports.column("is_input").unwrap().bool().unwrap();

        (0..self.io_ports.height()).filter_map(move |i| {
            if !is_input.get(i)? {
                Some((names.get(i)?.to_string(), cell_ids.get(i)?))
            } else {
                None
            }
        })
    }
}

/// Converts a u8 back to CellKind.
fn cell_kind_from_u8(v: u8) -> CellKind {
    match v {
        0 => CellKind::Buf,
        1 => CellKind::Not,
        2 => CellKind::And,
        3 => CellKind::Or,
        4 => CellKind::Xor,
        5 => CellKind::Mux,
        6 => CellKind::Adc,
        7 => CellKind::Aig,
        8 => CellKind::Eq,
        9 => CellKind::ULt,
        10 => CellKind::SLt,
        11 => CellKind::Shl,
        12 => CellKind::UShr,
        13 => CellKind::SShr,
        14 => CellKind::XShr,
        15 => CellKind::Mul,
        16 => CellKind::UDiv,
        17 => CellKind::UMod,
        18 => CellKind::SDivTrunc,
        19 => CellKind::SDivFloor,
        20 => CellKind::SModTrunc,
        21 => CellKind::SModFloor,
        22 => CellKind::Match,
        23 => CellKind::Assign,
        24 => CellKind::DLatchSr,
        25 => CellKind::ADLatch,
        26 => CellKind::Dff,
        27 => CellKind::Memory,
        28 => CellKind::IoBuf,
        29 => CellKind::Target,
        30 => CellKind::Other,
        31 => CellKind::Input,
        32 => CellKind::Output,
        33 => CellKind::Name,
        34 => CellKind::Debug,
        _ => CellKind::Other,
    }
}
