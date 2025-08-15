use std::{collections::{HashMap, HashSet}};

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

// Rename: build_precells -> collect_gate_cells, with clearer variable names
fn collect_gate_cells<'a>(design: &'a Design) -> (
    Vec<(CellRef<'a>, CellKind, Vec<Result<(usize, usize), Trit>>)>,
    HashMap<usize, usize>,
    HashMap<CellKind, Vec<usize>>,
) {
    let mut cell_entries = Vec::new();
    let mut cell_index_to_position = HashMap::new();
    let mut cells_by_kind: HashMap<CellKind, Vec<usize>> = HashMap::new();

    for cell in design.iter_cells() {
        let kind = cell_kind(&*cell.get());
        if !is_gate_kind(kind) { continue; }
        let mut input_connections: Vec<Result<(usize, usize), Trit>> = Vec::new();
        cell.visit(|net| {
            match design.find_cell(net) {
                Ok((src, bit)) => input_connections.push(Ok((src.debug_index(), bit))),
                Err(trit) => input_connections.push(Err(trit)),
            }
        });
        let position = cell_entries.len();
        cell_index_to_position.insert(cell.debug_index(), position);
        cells_by_kind.entry(kind).or_default().push(position);
        cell_entries.push((cell, kind, input_connections));
    }

    (cell_entries, cell_index_to_position, cells_by_kind)
}

// Rename: compatible_inputs -> are_inputs_compatible
fn are_inputs_compatible(
    pattern_inputs: &[Result<(usize, usize), Trit>],
    design_inputs: &[Result<(usize, usize), Trit>],
    pattern_to_design: &HashMap<usize, usize>,
) -> bool {
    if pattern_inputs.len() != design_inputs.len() { return false; }
    for (p_in, d_in) in pattern_inputs.iter().zip(design_inputs.iter()) {
        match (p_in, d_in) {
            (Err(a), Err(b)) => { if a != b { return false; } },
            (Ok((p_src, p_bit)), Ok((d_src, d_bit))) => {
                if let Some(&mapped_d_src) = pattern_to_design.get(p_src) {
                    if mapped_d_src != *d_src || p_bit != d_bit { return false; }
                }
            }
            _ => return false,
        }
    }
    true
}

// Rename: choose_next_needle -> choose_next_pattern_cell_index
fn choose_next_pattern_cell_index<'a>(
    pattern_cells: &[(CellRef<'a>, CellKind, Vec<Result<(usize, usize), Trit>>) ],
    mapped_pattern_indices: &HashSet<usize>,
    pattern_to_design: &HashMap<usize, usize>,
) -> Option<usize> {
    // Prefer cells whose inputs are all constants or mapped sources, to minimize branching
    for (i, (cref, _kind, inputs)) in pattern_cells.iter().enumerate() {
        if mapped_pattern_indices.contains(&cref.debug_index()) { continue; }
        let mut all_sources_mapped = true;
        for inp in inputs {
            if let Ok((src, _)) = inp {
                if !pattern_to_design.contains_key(src) { all_sources_mapped = false; break; }
            }
        }
        if all_sources_mapped { return Some(i); }
    }
    // fallback: first unmapped
    for (i, (cref, ..)) in pattern_cells.iter().enumerate() {
        if !mapped_pattern_indices.contains(&cref.debug_index()) { return Some(i); }
    }
    None
}

// Rename: backtrack -> backtrack_mappings, and rename parameters
fn backtrack_mappings<'a>(
    pattern_cells: &'a [(CellRef<'a>, CellKind, Vec<Result<(usize, usize), Trit>>) ],
    design_cells: &'a [(CellRef<'a>, CellKind, Vec<Result<(usize, usize), Trit>>) ],
    design_cells_by_kind: &HashMap<CellKind, Vec<usize>>,
    pattern_to_design: &mut HashMap<usize, usize>,
    used_design_indices: &mut HashSet<usize>,
    mappings_out: &mut Vec<HashMap<usize, usize>>,
) {
    if pattern_to_design.len() == pattern_cells.len() {
        mappings_out.push(pattern_to_design.clone());
        return;
    }

    let mapped_pattern_indices: HashSet<usize> = pattern_to_design.keys().copied().collect();
    let Some(next_pattern_idx) = choose_next_pattern_cell_index(pattern_cells, &mapped_pattern_indices, pattern_to_design) else { return; };
    let (pattern_cref, pattern_kind, pattern_inputs) = &pattern_cells[next_pattern_idx];

    let Some(design_candidate_positions) = design_cells_by_kind.get(pattern_kind) else { return; };

    for &design_pos in design_candidate_positions {
        let (design_cref, _d_kind, design_inputs) = &design_cells[design_pos];
        let pattern_key = pattern_cref.debug_index();
        let design_key = design_cref.debug_index();
        if used_design_indices.contains(&design_key) { continue; }
        if !are_inputs_compatible(pattern_inputs, design_inputs, pattern_to_design) { continue; }

        pattern_to_design.insert(pattern_key, design_key);
        used_design_indices.insert(design_key);
        backtrack_mappings(pattern_cells, design_cells, design_cells_by_kind, pattern_to_design, used_design_indices, mappings_out);
        used_design_indices.remove(&design_key);
        pattern_to_design.remove(&pattern_key);
    }
}

pub fn find_subgraphs(
    pattern: &Design,
    design: &Design,
) -> Vec<HashMap<usize, usize>> {

    let pattern_cell_types = count_cells_by_kind(pattern).into_iter().filter(|(kind, _)| is_gate_kind(*kind)).collect::<Vec<_>>();
    let design_cell_types = count_cells_by_kind(design).into_iter().filter(|(kind, _)| is_gate_kind(*kind)).collect::<Vec<_>>();

    // find the smallest cell kind in the design that is also in the pattern
    let anchor_kind = pattern_cell_types
        .iter()
        .filter_map(|(kind, _)| {
            design_cell_types.iter().find(|(d_kind, _)| d_kind == kind).map(|_| *kind)
        })
        .min_by_key(|kind| {
            design_cell_types.iter().find(|(d_kind, _)| d_kind == kind)
                .map(|(_, count)| *count)
        })
        .expect("No common cell kind found between pattern and design");

    // Build gate-only cell entries and buckets
    let (pattern_cells_all, _pattern_index_to_pos, _) = collect_gate_cells(pattern);
    let (design_cells_all, _design_index_to_pos, design_cells_by_kind) = collect_gate_cells(design);

    // Extract anchors
    let pattern_anchor_indices: Vec<usize> = pattern_cells_all
        .iter()
        .enumerate()
        .filter(|(_, entry)| { let (_, kind, _) = entry; *kind == anchor_kind })
        .map(|(i, _)| i)
        .collect();
    let design_anchor_indices: Vec<usize> = design_cells_all
        .iter()
        .enumerate()
        .filter(|(_, entry)| { let (_, kind, _) = entry; *kind == anchor_kind })
        .map(|(i, _)| i)
        .collect();

    let mut mappings = Vec::new();

    let p_anchor: Option<usize> = pattern_anchor_indices.first().map(|i| *i);

    if p_anchor.is_none() {
        log::warn!("Pattern has no anchor cells of kind {:?}", anchor_kind);
        return mappings; // No anchors means no matches
    }

    let p_anchor = p_anchor.unwrap();

    // for &p_anchor in &pattern_anchor_indices {
    for &d_anchor in &design_anchor_indices {
        let mut pattern_to_design_map: HashMap<usize, usize> = HashMap::new();
        let mut used_design: HashSet<usize> = HashSet::new();

        let pattern_key = pattern_cells_all[p_anchor].0.debug_index();
        let design_key = design_cells_all[d_anchor].0.debug_index();
        pattern_to_design_map.insert(pattern_key, design_key);
        used_design.insert(design_key);

        backtrack_mappings(
            &pattern_cells_all,
            &design_cells_all,
            &design_cells_by_kind,
            &mut pattern_to_design_map,
            &mut used_design,
            &mut mappings,
        );
    }
    // }

    mappings
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

        let needle_path = "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
        let needle_design = read_input(None, needle_path.to_string()).expect("Failed to read input design");
        let needle_name = get_name(&needle_path);

        // Find subgraphs using the chosen anchor kind
        let matches = find_subgraphs(&needle_design, &haystack_design);
        assert_eq!(matches.len(), 2, "Expected exactly two matches for {} with {}", needle_name, haystack_name);

    }
}