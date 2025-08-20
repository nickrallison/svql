use crate::SubgraphMatch;

/// A small, self-documenting signature component replacing tuple typing.
///
/// role: 0 = gate mapping; 1 = boundary (IO) binding
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct SigPart {
    role: u8,
    p_idx: usize,
    p_bit: usize,
    d_idx: usize,
    d_bit: usize,
}

// Alias to simplify clippy::type_complexity in dedupe signatures
type SigBoundary = Vec<SigPart>;

pub(crate) fn signature_with_boundary<'p, 'd>(m: &SubgraphMatch<'p, 'd>) -> SigBoundary {
    let gates = m.cell_mapping.iter().map(|(p, d)| SigPart {
        role: 0,
        p_idx: p.debug_index(),
        p_bit: 0,
        d_idx: d.debug_index(),
        d_bit: 0,
    });

    let boundaries = m
        .boundary_src_map
        .iter()
        .map(|((p_cell, p_bit), (d_cell, d_bit))| SigPart {
            role: 1,
            p_idx: p_cell.debug_index(),
            p_bit: *p_bit,
            d_idx: d_cell.debug_index(),
            d_bit: *d_bit,
        });

    let mut sig: SigBoundary = gates.chain(boundaries).collect();
    sig.sort_unstable();
    sig
}

pub(crate) fn signature_mapped_gate_set<'p, 'd>(m: &SubgraphMatch<'p, 'd>) -> Vec<usize> {
    let mut sig: Vec<usize> = m.cell_mapping.values().map(|d| d.debug_index()).collect();
    sig.sort_unstable();
    sig.dedup();
    sig
}
