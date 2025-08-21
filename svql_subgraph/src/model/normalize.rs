use super::{CellKind, Source};

pub(crate) fn is_commutative(kind: CellKind) -> bool {
    matches!(
        kind,
        CellKind::And | CellKind::Or | CellKind::Xor | CellKind::Eq
    )
}

pub(crate) fn normalize_commutative(inputs: &mut [Source]) {
    inputs.sort_by_key(stable_key);
}

fn stable_key(s: &Source) -> (u8, usize, usize) {
    match s {
        Source::Const(t) => (0, (*t as i8 as i32) as usize, 0),
        Source::Io(c, bit) => (1, c.index(), *bit),
        Source::Gate(c, bit) => (2, c.index(), *bit),
    }
}
