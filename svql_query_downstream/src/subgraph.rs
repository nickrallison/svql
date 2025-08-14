use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::{Cell, CellRef, Design, Trit};

use crate::cell_index::CellKind;

#[derive(Clone, Debug, PartialEq, Eq)]
enum InputSig {
    FromCell { src: usize, bit: usize },
    Const(Trit),
}

#[derive(Clone)]
struct PreCell<'a> {
    cref: CellRef<'a>,
    kind: CellKind,
    inputs: Vec<InputSig>,
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

fn build_precells<'a>(design: &'a Design) -> (Vec<PreCell<'a>>, HashMap<usize, usize>, HashMap<CellKind, Vec<usize>>) {
    let mut precells = Vec::new();
    let mut index_to_pos = HashMap::new();
    let mut by_kind: HashMap<CellKind, Vec<usize>> = HashMap::new();

    for cell in design.iter_cells() {
        let kind = cell_kind(&*cell.get());
        if !is_gate_kind(kind) { continue; }
        let mut inputs = Vec::new();
        cell.visit(|net| {
            match design.find_cell(net) {
                Ok((src, bit)) => inputs.push(InputSig::FromCell { src: src.debug_index(), bit }),
                Err(trit) => inputs.push(InputSig::Const(trit)),
            }
        });
        let pos = precells.len();
        index_to_pos.insert(cell.debug_index(), pos);
        by_kind.entry(kind).or_default().push(pos);
        precells.push(PreCell { cref: cell, kind, inputs });
    }

    (precells, index_to_pos, by_kind)
}

fn compatible_inputs(
    needle_inputs: &[InputSig],
    hay_inputs: &[InputSig],
    map_n2h: &HashMap<usize, usize>,
) -> bool {
    if needle_inputs.len() != hay_inputs.len() { return false; }
    for (n, h) in needle_inputs.iter().zip(hay_inputs.iter()) {
        match (n, h) {
            (InputSig::Const(a), InputSig::Const(b)) => { if a != b { return false; } },
            (InputSig::FromCell { src, bit }, InputSig::FromCell { src: hsrc, bit: hbit }) => {
                if let Some(&mapped_hsrc) = map_n2h.get(src) {
                    if mapped_hsrc != *hsrc || bit != hbit { return false; }
                } // else: source not mapped yet; cannot constrain
            }
            // constants must match constants; if needle has const and hay has FromCell, or vice versa, reject
            _ => return false,
        }
    }
    true
}

fn choose_next_needle(needle: &Vec<PreCell>, mapped: &HashSet<usize>, map_n2h: &HashMap<usize, usize>) -> Option<usize> {
    // Prefer cells whose inputs are all constants or mapped sources, to minimize branching
    for (i, pc) in needle.iter().enumerate() {
        if mapped.contains(&pc.cref.debug_index()) { continue; }
        let mut ok = true;
        for inp in &pc.inputs {
            if let InputSig::FromCell { src, .. } = inp {
                if !map_n2h.contains_key(src) { ok = false; break; }
            }
        }
        if ok { return Some(i); }
    }
    // fallback: first unmapped
    for (i, pc) in needle.iter().enumerate() {
        if !mapped.contains(&pc.cref.debug_index()) { return Some(i); }
    }
    None
}

fn backtrack<'a>(
    needle: &'a [PreCell<'a>],
    hay: &'a [PreCell<'a>],
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
    let Some(n_idx) = choose_next_needle(&needle.to_vec(), &mapped_needles, map_n2h) else { return; };
    let ncell = &needle[n_idx];

    let Some(hcands_pos) = hay_by_kind.get(&ncell.kind) else { return; };

    for &h_pos in hcands_pos {
        let hcell = &hay[h_pos];
        let n_key = ncell.cref.debug_index();
        let h_key = hcell.cref.debug_index();
        if used_hay.contains(&h_key) { continue; }
        if !compatible_inputs(&ncell.inputs, &hcell.inputs, map_n2h) { continue; }

        map_n2h.insert(n_key, h_key);
        used_hay.insert(h_key);
        backtrack(needle, hay, hay_by_kind, map_n2h, used_hay, out);
        used_hay.remove(&h_key);
        map_n2h.remove(&n_key);
    }
}

pub fn find_gate_subgraphs_by_anchor_kind(
    needle: &Design,
    haystack: &Design,
    anchor_kind: CellKind,
) -> Vec<HashMap<usize, usize>> {
    // Build gate-only precells and buckets
    let (n_precells_all, n_index_to_pos, _) = build_precells(needle);
    let (h_precells_all, _h_index_to_pos, h_by_kind) = build_precells(haystack);

    // Extract anchors
    let n_anchors: Vec<usize> = n_precells_all
        .iter()
        .enumerate()
        .filter(|(_i, pc)| pc.kind == anchor_kind)
        .map(|(i, _)| i)
        .collect();
    let h_anchors: Vec<usize> = h_precells_all
        .iter()
        .enumerate()
        .filter(|(_i, pc)| pc.kind == anchor_kind)
        .map(|(i, _)| i)
        .collect();

    let mut results = Vec::new();

    for &na in &n_anchors {
        for &ha in &h_anchors {
            let mut map_n2h: HashMap<usize, usize> = HashMap::new();
            let mut used_hay: HashSet<usize> = HashSet::new();

            let n_key = n_precells_all[na].cref.debug_index();
            let h_key = h_precells_all[ha].cref.debug_index();
            map_n2h.insert(n_key, h_key);
            used_hay.insert(h_key);

            backtrack(&n_precells_all, &h_precells_all, &h_by_kind, &mut map_n2h, &mut used_hay, &mut results);
        }
    }

    results
}
