use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::{Cell, CellRef, Design, Trit};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CellKind {
    Buf,
    Not,
    And,
    Or,
    Xor,
    Mux,
    Adc,
    Aig,
    Eq,
    ULt,
    SLt,
    Shl,
    UShr,
    SShr,
    XShr,
    Mul,
    UDiv,
    UMod,
    SDivTrunc,
    SDivFloor,
    SModTrunc,
    SModFloor,
    Match,
    Assign,
    Dff,
    Memory,
    IoBuf,
    Target,
    Other,
    Input,
    Output,
    Name,
    Debug,
}

impl From<&Cell> for CellKind {
    fn from(c: &Cell) -> Self {
        match c {
            Cell::Buf(..) => CellKind::Buf,
            Cell::Not(..) => CellKind::Not,
            Cell::And(..) => CellKind::And,
            Cell::Or(..) => CellKind::Or,
            Cell::Xor(..) => CellKind::Xor,
            Cell::Mux(..) => CellKind::Mux,
            Cell::Adc(..) => CellKind::Adc,
            Cell::Aig(..) => CellKind::Aig,
            Cell::Eq(..) => CellKind::Eq,
            Cell::ULt(..) => CellKind::ULt,
            Cell::SLt(..) => CellKind::SLt,
            Cell::Shl(..) => CellKind::Shl,
            Cell::UShr(..) => CellKind::UShr,
            Cell::SShr(..) => CellKind::SShr,
            Cell::XShr(..) => CellKind::XShr,
            Cell::Mul(..) => CellKind::Mul,
            Cell::UDiv(..) => CellKind::UDiv,
            Cell::UMod(..) => CellKind::UMod,
            Cell::SDivTrunc(..) => CellKind::SDivTrunc,
            Cell::SDivFloor(..) => CellKind::SDivFloor,
            Cell::SModTrunc(..) => CellKind::SModTrunc,
            Cell::SModFloor(..) => CellKind::SModFloor,
            Cell::Match(..) => CellKind::Match,
            Cell::Assign(..) => CellKind::Assign,
            Cell::Dff(..) => CellKind::Dff,
            Cell::Memory(..) => CellKind::Memory,
            Cell::IoBuf(..) => CellKind::IoBuf,
            Cell::Target(..) => CellKind::Target,
            Cell::Other(..) => CellKind::Other,
            Cell::Input(..) => CellKind::Input,
            Cell::Output(..) => CellKind::Output,
            Cell::Name(..) => CellKind::Name,
            Cell::Debug(..) => CellKind::Debug,
        }
    }
}

fn count_cells_by_kind(design: &Design) -> Vec<(CellKind, usize)> {
    let mut counts = HashMap::new();
    for cell in design.iter_cells() {
        let kind = cell_kind(&*cell.get());
        *counts.entry(kind).or_insert(0) += 1;
    }
    counts.into_iter().collect::<Vec<_>>()
}

fn cell_kind(c: &Cell) -> CellKind { CellKind::from(c) }

fn is_gate_kind(kind: CellKind) -> bool {
    matches!(
        kind,
        CellKind::Buf
            | CellKind::Not
            | CellKind::And
            | CellKind::Or
            | CellKind::Xor
            | CellKind::Mux
            | CellKind::Adc
            | CellKind::Aig
            | CellKind::Eq
            | CellKind::ULt
            | CellKind::SLt
            | CellKind::Shl
            | CellKind::UShr
            | CellKind::SShr
            | CellKind::XShr
            | CellKind::Mul
            | CellKind::UDiv
            | CellKind::UMod
            | CellKind::SDivTrunc
            | CellKind::SDivFloor
            | CellKind::SModTrunc
            | CellKind::SModFloor
            | CellKind::Dff // added
    )
}

fn build_precells<'a>(design: &'a Design) -> (
    Vec<(CellRef<'a>, CellKind, Vec<Result<(usize, usize), Trit>>)>,
    HashMap<usize, usize>,
    HashMap<CellKind, Vec<usize>>,
) {
    let mut precells = Vec::new();
    let mut index_to_pos = HashMap::new();
    let mut by_kind: HashMap<CellKind, Vec<usize>> = HashMap::new();

    for cell in design.iter_cells() {
        let kind = cell_kind(&*cell.get());
        if !is_gate_kind(kind) { continue; }
        let mut inputs: Vec<Result<(usize, usize), Trit>> = Vec::new();
        cell.visit(|net| {
            match design.find_cell(net) {
                Ok((src, bit)) => inputs.push(Ok((src.debug_index(), bit))),
                Err(trit) => inputs.push(Err(trit)),
            }
        });
        let pos = precells.len();
        index_to_pos.insert(cell.debug_index(), pos);
        by_kind.entry(kind).or_default().push(pos);
        precells.push((cell, kind, inputs));
    }

    (precells, index_to_pos, by_kind)
}

fn compatible_inputs(
    needle_inputs: &[Result<(usize, usize), Trit>],
    hay_inputs: &[Result<(usize, usize), Trit>],
    map_n2h: &HashMap<usize, usize>,
) -> bool {
    if needle_inputs.len() != hay_inputs.len() { return false; }
    for (n, h) in needle_inputs.iter().zip(hay_inputs.iter()) {
        match (n, h) {
            (Err(a), Err(b)) => { if a != b { return false; } },
            (Ok((src, bit)), Ok((hsrc, hbit))) => {
                if let Some(&mapped_hsrc) = map_n2h.get(src) {
                    if mapped_hsrc != *hsrc || bit != hbit { return false; }
                }
            }
            _ => return false,
        }
    }
    true
}

fn choose_next_needle<'a>(
    needle: &[(CellRef<'a>, CellKind, Vec<Result<(usize, usize), Trit>>) ],
    mapped: &HashSet<usize>,
    map_n2h: &HashMap<usize, usize>,
) -> Option<usize> {
    // Prefer cells whose inputs are all constants or mapped sources, to minimize branching
    for (i, (cref, _kind, inputs)) in needle.iter().enumerate() {
        if mapped.contains(&cref.debug_index()) { continue; }
        let mut ok = true;
        for inp in inputs {
            if let Ok((src, _)) = inp {
                if !map_n2h.contains_key(src) { ok = false; break; }
            }
        }
        if ok { return Some(i); }
    }
    // fallback: first unmapped
    for (i, (cref, ..)) in needle.iter().enumerate() {
        if !mapped.contains(&cref.debug_index()) { return Some(i); }
    }
    None
}

fn backtrack<'a>(
    needle: &'a [(CellRef<'a>, CellKind, Vec<Result<(usize, usize), Trit>>) ],
    hay: &'a [(CellRef<'a>, CellKind, Vec<Result<(usize, usize), Trit>>) ],
    hay_by_kind: &HashMap<CellKind, Vec<usize>>,
    map_n2h: &mut HashMap<usize, usize>,
    used_hay: &mut HashSet<usize>,
    out: &mut Vec<HashMap<usize, usize>>,
) {
    if map_n2h.len() == needle.len() {
        out.push(map_n2h.clone());
        return;
    }

    let mapped_needles: HashSet<usize> = map_n2h.keys().copied().collect();
    let Some(n_idx) = choose_next_needle(needle, &mapped_needles, map_n2h) else { return; };
    let (n_cref, n_kind, n_inputs) = &needle[n_idx];

    let Some(hcands_pos) = hay_by_kind.get(n_kind) else { return; };

    for &h_pos in hcands_pos {
        let (h_cref, _h_kind, h_inputs) = &hay[h_pos];
        let n_key = n_cref.debug_index();
        let h_key = h_cref.debug_index();
        if used_hay.contains(&h_key) { continue; }
        if !compatible_inputs(n_inputs, h_inputs, map_n2h) { continue; }

        map_n2h.insert(n_key, h_key);
        used_hay.insert(h_key);
        backtrack(needle, hay, hay_by_kind, map_n2h, used_hay, out);
        used_hay.remove(&h_key);
        map_n2h.remove(&n_key);
    }
}

pub fn find_subgraphs(
    needle: &Design,
    haystack: &Design,
) -> Vec<HashMap<usize, usize>> {

    let needle_cell_types = count_cells_by_kind(needle);
    let haystack_cell_types = count_cells_by_kind(haystack);

    // find the smallest cell kind in the haystack that is also in the needle
    let anchor_kind = needle_cell_types
        .iter()
        .filter_map(|(kind, _)| {
            haystack_cell_types.iter().find(|(hkind, _)| hkind == kind).map(|_| *kind)
        })
        .min_by_key(|kind| {
            haystack_cell_types.iter().find(|(hkind, _)| hkind == kind)
                .map_or(usize::MAX, |(_, count)| *count)
        })
        .expect("No common cell kind found between needle and haystack");

    // Build gate-only precells and buckets
    let (n_precells_all, _n_index_to_pos, _) = build_precells(needle);
    let (h_precells_all, _h_index_to_pos, h_by_kind) = build_precells(haystack);

    // Extract anchors
    let n_anchors: Vec<usize> = n_precells_all
        .iter()
        .enumerate()
        .filter(|(_i, pc)| pc.1 == anchor_kind)
        .map(|(i, _)| i)
        .collect();
    let h_anchors: Vec<usize> = h_precells_all
        .iter()
        .enumerate()
        .filter(|(_i, pc)| pc.1 == anchor_kind)
        .map(|(i, _)| i)
        .collect();

    let mut results = Vec::new();

    for &na in &n_anchors {
        for &ha in &h_anchors {
            let mut map_n2h: HashMap<usize, usize> = HashMap::new();
            let mut used_hay: HashSet<usize> = HashSet::new();

            let n_key = n_precells_all[na].0.debug_index();
            let h_key = h_precells_all[ha].0.debug_index();
            map_n2h.insert(n_key, h_key);
            used_hay.insert(h_key);

            backtrack(&n_precells_all, &h_precells_all, &h_by_kind, &mut map_n2h, &mut used_hay, &mut results);
        }
    }

    results
}

#[cfg(test)]
mod tests {

    use crate::{get_name, read_input};

    use super::*;

    #[test]
    fn test_many_regs() {
        let haystack_path = "examples/patterns/security/access_control/locked_reg/json/many_locked_regs.json";
        let haystack_design = read_input(None, haystack_path.to_string()).expect("Failed to read input design");
        let haystack_name = get_name(&haystack_path);

        let needle_path = "examples/patterns/security/access_control/locked_reg/json/sync_mux.json";
        let needle_design = read_input(None, needle_path.to_string()).expect("Failed to read input design");
        let needle_name = get_name(&needle_path);

        // Find subgraphs using the chosen anchor kind
        let matches = find_subgraphs(&needle_design, &haystack_design);
        assert_eq!(matches.len(), 2, "Expected exactly two matches for {} with {}", needle_name, haystack_name);

    }
}