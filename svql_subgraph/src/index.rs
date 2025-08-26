use std::collections::HashMap;

use prjunnamed_netlist::{Cell, CellRef, Design};

use crate::cell::{CellKind, Source, net_to_source};

#[derive(Clone, Debug)]
pub(super) struct Index<'a> {
    /// Cells of design in topological order (Name cells filtered out)
    cells_topo: Vec<CellRef<'a>>,
    by_kind: HashMap<CellKind, Vec<CellRef<'a>>>,
    /// For each driver (key), the list of (sink, sink_pin_idx)
    reverse_cell_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>>,

    /// Pre-computed source information for each cell: sources[sink][pin_idx] = Source
    cell_sources: HashMap<CellRef<'a>, Vec<Source<'a>>>,

    /// fanout membership: driver -> (sink -> bitmask_of_pins)
    fanout_map: HashMap<CellRef<'a>, HashMap<CellRef<'a>, u32>>,
}

impl<'a> Index<'a> {
    #[contracts::debug_ensures(ret.gate_count() <= design.iter_cells().count())]
    pub(super) fn build(design: &'a Design) -> Self {
        tracing::event!(
            tracing::Level::TRACE,
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

        tracing::event!(
            tracing::Level::TRACE,
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
            tracing::event!(
                tracing::Level::TRACE,
                "Index::build: by_kind {:?} -> {}",
                k,
                v.len()
            );
        }

        let mut reverse_cell_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>> =
            HashMap::new();
        let mut cell_sources: HashMap<CellRef<'a>, Vec<Source<'a>>> = HashMap::new();
        let mut fanout_map: HashMap<CellRef<'a>, HashMap<CellRef<'a>, u32>> = HashMap::new();

        // Pre-compute source information for all cells (as sinks)
        for cell_ref in cells_topo.iter() {
            let mut sources: Vec<Source<'a>> = Vec::new();
            cell_ref.visit(|net| {
                sources.push(net_to_source(design, net));
            });
            cell_sources.insert(*cell_ref, sources);
        }

        // Build reverse lookups and O(1) fanout membership map
        for sink_ref in cells_topo.iter() {
            let sources = cell_sources.get(sink_ref).unwrap();
            for (sink_pin_idx, source) in sources.iter().enumerate() {
                let driver_cell = match source {
                    Source::Gate(cell_ref, _src_bit) => Some(*cell_ref),
                    Source::Io(cell_ref, _src_bit) => Some(*cell_ref),
                    Source::Const(_trit) => None,
                };

                if let Some(driver) = driver_cell {
                    // reverse list (driver -> vec of (sink, pin))
                    reverse_cell_lookup
                        .entry(driver)
                        .or_default()
                        .push((*sink_ref, sink_pin_idx));

                    // fanout_map (driver -> map sink -> bitmask)
                    let mask = 1u32 << sink_pin_idx;
                    let entry = fanout_map.entry(driver).or_default();
                    entry
                        .entry(*sink_ref)
                        .and_modify(|m| *m |= mask)
                        .or_insert(mask);
                }
            }
        }

        tracing::event!(
            tracing::Level::TRACE,
            "Index::build: reverse_cell_lookup keys={} (drivers).",
            reverse_cell_lookup.len()
        );
        tracing::event!(
            tracing::Level::TRACE,
            "Index::build: fanout_map keys={} (drivers).",
            fanout_map.len()
        );

        Index {
            cells_topo,
            by_kind,
            reverse_cell_lookup,
            cell_sources,
            fanout_map,
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
        tracing::event!(
            tracing::Level::TRACE,
            "Index::get_by_kind: {:?} -> {}",
            kind,
            slice.len()
        );
        slice
    }

    pub(super) fn get_cells_topo(&self) -> &[CellRef<'a>] {
        self.cells_topo.as_slice()
    }

    pub(super) fn get_fanouts(&self, cell: CellRef<'a>) -> &[(CellRef<'a>, usize)] {
        let slice = self
            .reverse_cell_lookup
            .get(&cell)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        tracing::event!(
            tracing::Level::TRACE,
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

    /// True if `driver` has any fanout edge to `sink` (any input pin).
    pub(super) fn has_fanout_to(&self, driver: CellRef<'a>, sink: CellRef<'a>) -> bool {
        self.fanout_map
            .get(&driver)
            .and_then(|m| m.get(&sink))
            .is_some()
    }

    /// True if `driver` feeds `sink` specifically at `pin_idx`.
    pub(super) fn has_fanout_to_pin(
        &self,
        driver: CellRef<'a>,
        sink: CellRef<'a>,
        pin_idx: usize,
    ) -> bool {
        self.fanout_map
            .get(&driver)
            .and_then(|m| m.get(&sink))
            .is_some_and(|mask| (mask & (1u32 << pin_idx)) != 0)
    }

    /// The unique driver of a given sink input pin, if any (Gate/Io sources only).
    pub(super) fn driver_of_sink_pin(
        &self,
        sink: CellRef<'a>,
        pin_idx: usize,
    ) -> Option<CellRef<'a>> {
        let src = self.get_cell_sources(sink).get(pin_idx)?;
        match src {
            Source::Gate(c, _) | Source::Io(c, _) => Some(*c),
            Source::Const(_) => None,
        }
    }

    /// All drivers of a sink across all pins (Gate/Io only), duplicates removed.
    pub(super) fn drivers_of_sink_all_pins(&self, sink: CellRef<'a>) -> Vec<CellRef<'a>> {
        let mut out: Vec<CellRef<'a>> = self
            .get_cell_sources(sink)
            .iter()
            .filter_map(|src| match src {
                Source::Gate(c, _) | Source::Io(c, _) => Some(*c),
                Source::Const(_) => None,
            })
            .collect();

        out.sort_by_key(|c| c.debug_index());
        out.dedup();
        out
    }
}
