use std::collections::HashMap;

use prjunnamed_netlist::{CellRef, Design};

use crate::model::{CellKind, CellWrapper, Source};

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
        let mut by_kind: HashMap<CellKind, Vec<CellWrapper<'a>>> = HashMap::new();
        let cells_topo: Vec<CellWrapper<'a>> = design
            .iter_cells_topo()
            .rev()
            .map(CellWrapper::new)
            .filter(|cell| !matches!(cell.kind, CellKind::Name))
            .collect();
        let gates: Vec<CellWrapper<'a>> = cells_topo
            .iter()
            .filter(|cell| cell.kind.is_gate())
            .cloned()
            .collect();

        for cell_wrapper in gates.into_iter() {
            by_kind
                .entry(cell_wrapper.kind)
                .or_default()
                .push(cell_wrapper);
        }

        let mut reverse_cell_lookup: HashMap<CellRef<'a>, Vec<(CellRef<'a>, usize)>> =
            HashMap::new();

        // reverse cell_lookup
        for cell_wrapper in cells_topo.iter() {
            let pins = &cell_wrapper.pins;
            for pin in pins {
                let pin_driver = match pin {
                    Source::Gate(cell_ref, id) => Some((cell_ref, id)),
                    Source::Io(cell_ref, id) => Some((cell_ref, id)),
                    Source::Const(_trit) => None,
                };
                if let Some((pin_driver, id)) = pin_driver {
                    reverse_cell_lookup
                        .entry(*pin_driver)
                        .or_default()
                        .push((cell_wrapper.cref(), *id));
                }
            }
        }

        Index {
            cells_topo,
            by_kind,
            reverse_cell_lookup,
        }
    }

    pub(super) fn of_kind(&self, k: CellKind) -> &[CellWrapper<'a>] {
        self.by_kind.get(&k).map(|v| v.as_slice()).unwrap_or(&[])
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
        self.by_kind.get(&kind).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub(super) fn get_cells_topo(&self) -> &[CellWrapper<'a>] {
        &self.cells_topo
    }
    pub(super) fn get_outputs(&self) -> Vec<CellWrapper<'a>> {
        self.cells_topo
            .iter()
            .filter(|c| c.kind.is_output())
            .cloned()
            .collect()
    }

    pub(super) fn get_fanouts(&self, cell: CellRef<'a>) -> &[(CellRef<'a>, usize)] {
        self.reverse_cell_lookup
            .get(&cell)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}
