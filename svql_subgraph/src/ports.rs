use prjunnamed_netlist::{Design, Net, Trit};

use crate::cell::CellWrapper;

use super::cell::{CellKind, is_gate_cell_ref};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Source<'a> {
    Gate(CellWrapper<'a>, usize),
    Io(CellWrapper<'a>, usize),
    Const(Trit),
}

#[derive(Clone, Debug)]
pub(super) struct CellPins<'a> {
    pub(super) inputs: Vec<Source<'a>>,
}

pub(super) fn is_commutative(kind: CellKind) -> bool {
    matches!(
        kind,
        CellKind::And | CellKind::Or | CellKind::Xor | CellKind::Eq
    )
}

pub(super) fn extract_pins<'a>(cref: CellWrapper<'a>) -> CellPins<'a> {
    let mut inputs: Vec<Source<'a>> = Vec::new();
    cref.visit(|net| {
        inputs.push(net_to_source(cref.design(), net));
    });
    CellPins { inputs }
}

fn net_to_source<'a>(design: &'a Design, net: Net) -> Source<'a> {
    match design.find_cell(net) {
        Ok((src, bit)) => {
            if is_gate_cell_ref(src) {
                Source::Gate(src.into(), bit)
            } else {
                Source::Io(src.into(), bit)
            }
        }
        Err(trit) => Source::Const(trit),
    }
}

pub(super) fn normalize_commutative<'a>(inputs: &mut [Source<'a>]) {
    inputs.sort_by(|a, b| stable_key(a).cmp(&stable_key(b)));
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
    use prjunnamed_netlist::Design;

    use super::*;

    lazy_static::lazy_static! {
        static ref SDFFE: Design = crate::util::load_design_from("examples/patterns/basic/ff/verilog/sdffe.v").unwrap();
    }

    #[test]
    fn can_extract_some_pins() {
        let d = &SDFFE;
        for c in d.iter_cells() {
            let _pins = extract_pins(c.into());
        }
    }

    #[test]
    fn commutative_sort_is_stable() {
        let d = &SDFFE;
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
