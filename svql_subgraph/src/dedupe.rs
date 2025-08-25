use crate::SubgraphMatch;

// Alias retained in case we expand later
type SigDesignGateSet = Vec<usize>;

pub(crate) fn signature_mapped_gate_set<'p, 'd>(m: &SubgraphMatch<'p, 'd>) -> SigDesignGateSet {
    let mut sig: Vec<usize> = m.cell_mapping.values().map(|d| d.debug_index()).collect();
    sig.sort_unstable();
    sig.dedup();
    sig
}
