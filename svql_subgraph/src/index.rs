use std::collections::HashMap;

use prjunnamed_netlist::{Cell, CellRef, Design};

use crate::cell::{CellKind, Source, net_to_source};
use tracing::trace;

#[derive(Clone, Debug)]
pub(super) struct Index<'a> {
    /// Cells of design in topological order
    cells_topo: Vec<CellRef<'a>>,
    by_kind: HashMap<CellKind, Vec<CellRef<'a>>>,
    reverse_cell_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>>,
    /// Pre-computed source information for each cell
    cell_sources: HashMap<CellRef<'a>, Vec<Source<'a>>>,
}

impl<'a> Index<'a> {
    #[contracts::debug_ensures(ret.gate_count() <= design.iter_cells().count())]
    pub(super) fn build(design: &'a Design) -> Self {
        trace!(
            "Index::build: start. design cells={}",
            design.iter_cells().count()
        );

        let mut by_kind: HashMap<CellKind, Vec<CellRef<'a>>> = HashMap::new();
        let cells_topo: Vec<CellRef<'a>> = design
            .iter_cells_topo()
            .rev()
            .filter(|cell_ref| {
                let kind = CellKind::from(cell_ref.get().as_ref());
                !matches!(kind, CellKind::Name)
            })
            .collect();
        trace!(
            "Index::build: cells_topo len={} (excluding Name). gate_count={}",
            cells_topo.len(),
            cells_topo
                .iter()
                .filter(|c| CellKind::from(c.get().as_ref()).is_gate())
                .count()
        );

        for cell_ref in cells_topo.iter().cloned() {
            let kind = CellKind::from(cell_ref.get().as_ref());
            by_kind.entry(kind).or_default().push(cell_ref);
        }

        for (k, v) in by_kind.iter() {
            trace!("Index::build: by_kind {:?} -> {}", k, v.len());
        }

        let mut reverse_cell_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>> =
            HashMap::new();
        let mut cell_sources: HashMap<CellRef<'a>, Vec<Source<'a>>> = HashMap::new();

        // Pre-compute source information for all cells
        for cell_ref in cells_topo.iter() {
            let mut sources: Vec<Source<'a>> = Vec::new();
            cell_ref.visit(|net| {
                sources.push(net_to_source(design, net));
            });
            cell_sources.insert(*cell_ref, sources);
        }

        // Build reverse lookup based on pre-computed sources
        for sink_ref in cells_topo.iter() {
            let sources = cell_sources.get(sink_ref).unwrap();
            for (sink_pin_idx, source) in sources.iter().enumerate() {
                let driver = match source {
                    Source::Gate(cell_ref, _src_bit) => Some(*cell_ref),
                    Source::Io(cell_ref, _src_bit) => Some(*cell_ref),
                    Source::Const(_trit) => None,
                };
                if let Some(driver_cell) = driver {
                    // Store the SINK'S input pin index (sink_pin_idx), not the source bit index.
                    reverse_cell_lookup
                        .entry(driver_cell)
                        .or_default()
                        .push((*sink_ref, sink_pin_idx));
                }
            }
        }

        trace!(
            "Index::build: reverse_cell_lookup keys={} (drivers).",
            reverse_cell_lookup.len()
        );

        Index {
            cells_topo,
            by_kind,
            reverse_cell_lookup,
            cell_sources,
        }
    }

    pub(super) fn gate_count(&self) -> usize {
        self.cells_topo
            .iter()
            .filter(|c| CellKind::from(c.get().as_ref()).is_gate())
            .count()
    }

    pub(super) fn get_by_kind(&self, kind: CellKind) -> &[CellRef<'a>] {
        let slice = self.by_kind.get(&kind).map(|v| v.as_slice()).unwrap_or(&[]);
        trace!("Index::get_by_kind: {:?} -> {}", kind, slice.len());
        slice
    }

    pub(super) fn get_cells_topo(&self) -> &[CellRef<'a>] {
        &self.cells_topo
    }

    pub(super) fn get_fanouts(&self, cell: CellRef<'a>) -> &[(CellRef<'a>, usize)] {
        let slice = self
            .reverse_cell_lookup
            .get(&cell)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        trace!(
            "Index::get_fanouts: driver #{} -> {} sinks",
            cell.debug_index(),
            slice.len()
        );
        slice
    }

    pub(super) fn get_cell_sources(&self, cell: CellRef<'a>) -> &[Source<'a>] {
        self.cell_sources
            .get(&cell)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub(super) fn get_cell_kind(&self, cell: CellRef<'a>) -> CellKind {
        CellKind::from(cell.get().as_ref())
    }

    pub(super) fn get_cell_input_name(&self, cell: CellRef<'a>) -> Option<&'a str> {
        match cell.get() {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub(super) fn get_cell_output_name(&self, cell: CellRef<'a>) -> Option<&'a str> {
        match cell.get() {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub(super) fn cell_summary(&self, cell: CellRef<'a>) -> String {
        let kind = self.get_cell_kind(cell);
        let iname = self.get_cell_input_name(cell).unwrap_or("");
        let oname = self.get_cell_output_name(cell).unwrap_or("");
        let n = if !iname.is_empty() { iname } else { oname };
        if n.is_empty() {
            format!("#{} {:?}", cell.debug_index(), kind)
        } else {
            format!("#{} {:?}({})", cell.debug_index(), kind, n)
        }
    }
}
