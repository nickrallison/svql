use std::collections::HashMap;

use prjunnamed_netlist::{CellRef, Design};

use crate::model::{CellKind, CellWrapper, Source};
use log::trace;

#[derive(Clone, Debug)]
pub(super) struct Index<'a> {
    /// Cells of design in topological order
    cells_topo: Vec<CellWrapper<'a>>,
    by_kind: HashMap<CellKind, Vec<CellWrapper<'a>>>,
    reverse_cell_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>>,
}

impl<'a> Index<'a> {
    #[contracts::debug_ensures(ret.gate_count() <= design.iter_cells().count())]
    pub(super) fn build(design: &'a Design) -> Self {
        trace!(
            "Index::build: start. design cells={}",
            design.iter_cells().count()
        );

        let mut by_kind: HashMap<CellKind, Vec<CellWrapper<'a>>> = HashMap::new();
        let cells_topo: Vec<CellWrapper<'a>> = design
            .iter_cells_topo()
            .rev()
            .map(CellWrapper::new)
            .filter(|cell| !matches!(cell.kind, CellKind::Name))
            .collect();
        trace!(
            "Index::build: cells_topo len={} (excluding Name). gate_count={}",
            cells_topo.len(),
            cells_topo.iter().filter(|c| c.kind.is_gate()).count()
        );

        for cell_wrapper in cells_topo.iter().cloned() {
            by_kind
                .entry(cell_wrapper.kind)
                .or_default()
                .push(cell_wrapper);
        }

        for (k, v) in by_kind.iter() {
            trace!("Index::build: by_kind {:?} -> {}", k, v.len());
        }

        let mut reverse_cell_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>> =
            HashMap::new();

        for sink_wrapper in cells_topo.iter() {
            for (sink_pin_idx, pin) in sink_wrapper.pins.iter().enumerate() {
                let driver = match pin {
                    Source::Gate(cell_ref, _src_bit) => Some(*cell_ref),
                    Source::Io(cell_ref, _src_bit) => Some(*cell_ref),
                    Source::Const(_trit) => None,
                };
                if let Some(driver_cell) = driver {
                    // Store the SINK'S input pin index (sink_pin_idx), not the source bit index.
                    reverse_cell_lookup
                        .entry(driver_cell)
                        .or_default()
                        .push((sink_wrapper.cref(), sink_pin_idx));
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
        }
    }

    pub(super) fn of_kind(&self, k: CellKind) -> &[CellWrapper<'a>] {
        let slice = self.by_kind.get(&k).map(|v| v.as_slice()).unwrap_or(&[]);
        trace!("Index::of_kind: {:?} -> {}", k, slice.len());
        slice
    }

    pub(super) fn gate_count(&self) -> usize {
        self.cells_topo.iter().filter(|c| c.kind.is_gate()).count()
    }

    pub(super) fn nodes(&self) -> &[CellWrapper<'a>] {
        &self.cells_topo
    }

    /// Deterministic owned iteration over kinds.
    pub(super) fn by_kind_iter(&self) -> Vec<(CellKind, Vec<CellWrapper<'a>>)> {
        let mut items: Vec<(CellKind, Vec<CellWrapper<'a>>)> =
            self.by_kind.iter().map(|(k, v)| (*k, v.clone())).collect();
        items.sort_by_key(|kn| kn.0);
        items
    }

    pub(super) fn get_by_kind(&self, kind: CellKind) -> &[CellWrapper<'a>] {
        let slice = self.by_kind.get(&kind).map(|v| v.as_slice()).unwrap_or(&[]);
        trace!("Index::get_by_kind: {:?} -> {}", kind, slice.len());
        slice
    }

    pub(super) fn get_cells_topo(&self) -> &[CellWrapper<'a>] {
        &self.cells_topo
    }

    pub(super) fn get_outputs(&self) -> Vec<CellWrapper<'a>> {
        let v: Vec<CellWrapper<'a>> = self
            .cells_topo
            .iter()
            .filter(|c| c.kind.is_output())
            .cloned()
            .collect();
        trace!("Index::get_outputs -> {}", v.len());
        v
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
}
