use prjunnamed_netlist::{CellRef, Trit};

use crate::subgraph::cell_kind::CellWrapper;

use super::cell_kind::{is_gate_cell_ref, CellKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PinKind {
    Data(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Source<'a> {
    Gate(CellWrapper<'a>, usize),
    Io(CellWrapper<'a>, usize),
    Const(Trit),
}

#[derive(Clone, Debug)]
pub(super) struct CellPins<'a> {
    pub(super) kind: CellKind,
    pub(super) inputs: Vec<(PinKind, Source<'a>)>,
}

pub(super) fn is_commutative(kind: CellKind) -> bool {
    matches!(kind, CellKind::And | CellKind::Or | CellKind::Xor | CellKind::Eq)
}

pub(super) fn extract_pins<'a>(cref: CellWrapper<'a>) -> CellPins<'a> {
    let kind = CellKind::from(cref.get().as_ref());
    let mut idx = 0usize;
    let mut inputs: Vec<(PinKind, Source<'a>)> = Vec::new();
    cref.visit(|net| {
        let pin = PinKind::Data(idx);
        idx += 1;
        match cref.design().find_cell(net) {
            Ok((src, bit)) => {
                if is_gate_cell_ref(src) {
                    inputs.push((pin, Source::Gate(src.into(), bit)));
                } else {
                    inputs.push((pin, Source::Io(src.into(), bit)));
                }
            }
            Err(trit) => inputs.push((pin, Source::Const(trit))),
        }
    });
    CellPins { kind, inputs }
}

pub(super) fn normalize_commutative<'a>(inputs: &mut [(PinKind, Source<'a>)]) {
    inputs.sort_by(|a, b| stable_key(&a.1).cmp(&stable_key(&b.1)));
}

fn stable_key<'a>(s: &Source<'a>) -> (u8, usize, usize) {
    match s {
        Source::Const(t) => (0, (*t as i8 as i32) as usize, 0),
        Source::Io(c, bit) => (1, c.debug_index(), *bit),
        Source::Gate(c, bit) => (2, c.debug_index(), *bit),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{Driver};
    use crate::util::load_driver_from;

    lazy_static::lazy_static! {
        static ref SDFFE: (Driver, String) = load_driver_from("examples/patterns/basic/ff/sdffe.v");
    }

    #[test]
    fn can_extract_some_pins() {
        let d = SDFFE.0.design_as_ref() ;
        for c in d.iter_cells() {
            let _pins = extract_pins(c.into());
        }
    }

    #[test]
    fn commutative_sort_is_stable() {
        let d = SDFFE.0.design_as_ref();
        for c in d.iter_cells() {
            let mut pins1 = extract_pins(c.into()).inputs;
            let mut pins2 = extract_pins(c.into()).inputs;
            normalize_commutative(&mut pins1);
            normalize_commutative(&mut pins2);
            assert_eq!(pins1.len(), pins2.len());
            assert_eq!(pins1, pins2);
        }
    }
}