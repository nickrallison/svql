use std::collections::{HashMap, HashSet};
use std::borrow::Cow;

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

#[derive(Clone, Default)]
pub struct SubgraphMatch<'p, 'd> {
    pub cell_mapping: HashMap<CellRef<'p>, CellRef<'d>>,
    pub pat_input_cells: Vec<InputCell<'p>>,
    pub pat_output_cells: Vec<OutputCell<'p>>,
}

impl<'p, 'd> SubgraphMatch<'p, 'd> {
    pub fn len(&self) -> usize { self.cell_mapping.len() }
    pub fn is_empty(&self) -> bool { self.cell_mapping.is_empty() }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, CellRef<'p>, CellRef<'d>> {
        self.cell_mapping.iter()
    }
}

impl std::fmt::Debug for SubgraphMatch<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Render deterministic, stable identifiers using debug_index() for CellRef
        let mut mapping: Vec<((usize, Cell), (usize, Cell))> = self
            .cell_mapping
            .iter()
            .map(|(p, d)| ((p.debug_index(), p.get().as_ref().clone()), (d.debug_index(), d.get().as_ref().clone())))
            .collect();
        mapping.sort_by(|a, b| a.0.0.cmp(&b.0.0));

        let mut inputs: Vec<InputCell> = self.pat_input_cells.clone();
        let mut outputs: Vec<OutputCell> = self.pat_output_cells.clone();

        f.debug_struct("SubgraphMatch")
            .field("cell_mapping", &mapping)
            .field("pat_input_cells", &inputs)
            .field("pat_output_cells", &outputs)
            .finish()
    }
}

#[derive(Clone)]
pub struct InputCell<'p> {
    pub cref: CellRef<'p>,
}

impl<'p> InputCell<'p> {
    pub fn name(&self) -> Option<&'p str> {
        match self.cref.get() {
            Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn get_gates(&self) -> Vec<CellRef<'p>> {
        if matches!(self.cref.get().as_ref(), Cell::Input(_, _)) {
            let fanout = get_fanout(self.cref.design(), self.cref);
            fanout
        } else {
            vec![]
        }
    }
}

impl std::fmt::Debug for InputCell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InputCell")
            .field(&self.name().unwrap_or("<unnamed>"))
            .field(&self.cref.debug_index())
            .field(self.cref.get().as_ref())
            .finish()
    }
}

pub fn get_fanout<'a>(
    design: &'a Design,
    cell: CellRef<'a>,
) -> Vec<CellRef<'a>> {
    let mut fanout: Vec<CellRef<'a>> = Vec::new();

    for dest in design.iter_cells() {
        // Skip self to avoid self-loops in fanout
        if dest == cell {
            continue;
        }

        let mut driven_by_cell = false;
        dest.visit(|net| {
            if driven_by_cell {
                return; // already found a connection from `cell` to `dest`
            }
            if let Ok((src, _bit)) = design.find_cell(net) {
                if src == cell {
                    driven_by_cell = true;
                }
            }
        });

        if driven_by_cell {
            fanout.push(dest);
        }
    }

    fanout
}

// Helpers: return names and CellRefs, not cloned Cells
fn get_input_cells<'a>(design: &'a Design) -> Vec<InputCell<'a>> {
    design
        .iter_cells()
        .filter(|cell_ref| matches!(cell_ref.get().as_ref(), Cell::Input(_, _)))
        .map(|cref| InputCell { cref })
        .collect()
}

#[derive(Clone)]
pub struct OutputCell<'p> {
    pub cref: CellRef<'p>,
}

impl<'p> OutputCell<'p> {
    pub fn name(&self) -> Option<&'p str> {
        match self.cref.get() {
            Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn get_gate(&self) -> CellRef<'p> {
        let mut source: Option<CellRef<'p>> = None;
        if matches!(self.cref.get().as_ref(), Cell::Output(_, _)) {
            self.cref.visit(|net| {
                if let Ok((src, _bit)) = self.cref.design().find_cell(net) {
                    source = Some(src);
                }
            });
        }
        source.expect("Output cell should have a driving source")
    }
}

impl std::fmt::Debug for OutputCell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OutputCell")
            .field(&self.name().unwrap_or("<unnamed>"))
            .field(&self.cref.debug_index())
            .field(self.cref.get().as_ref())
            .finish()
    }
}

fn get_output_cells<'a>(design: &'a Design) -> Vec<OutputCell<'a>> {
    design
        .iter_cells()
        .filter(|cell_ref| matches!(cell_ref.get().as_ref(), Cell::Output(_, _)))
        .map(|cref| OutputCell { cref })
        .collect()
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

fn is_matchable_kind(kind: CellKind) -> bool {
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
            | CellKind::Dff
            // | CellKind::Input
            // | CellKind::Output
            // | CellKind::IoBuf
    )
}

// Collect only gate cells and organize them keyed by CellRef, with a by-kind index of CellRefs
fn collect_matchable_cells<'a>(design: &'a Design) -> (
    HashMap<CellRef<'a>, (CellKind, Vec<Result<(CellRef<'a>, usize), Trit>>)>,
    HashMap<CellKind, Vec<CellRef<'a>>>,
) {
    let mut cell_map: HashMap<CellRef<'a>, (CellKind, Vec<Result<(CellRef<'a>, usize), Trit>>)> =
        HashMap::new();
    let mut cells_by_kind: HashMap<CellKind, Vec<CellRef<'a>>> = HashMap::new();

    for cell in design.iter_cells() {
        let kind = cell_kind(&*cell.get());
        if !is_matchable_kind(kind) {
            continue;
        }

        let mut input_connections: Vec<Result<(CellRef<'a>, usize), Trit>> = Vec::new();
        cell.visit(|net| {
            match design.find_cell(net) {
                Ok((src, bit)) => input_connections.push(Ok((src, bit))),
                Err(trit) => input_connections.push(Err(trit)),
            }
        });

        let cref = cell;
        cells_by_kind.entry(kind).or_default().push(cref);
        cell_map.insert(cref, (kind, input_connections));
    }

    (cell_map, cells_by_kind)
}

fn get_pattern_io_cells<'p>(
    pattern: &'p Design,
) -> (Vec<InputCell<'p>>, Vec<OutputCell<'p>>) {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    for cref in pattern.iter_cells() {
        match cref.get().as_ref() {
            Cell::Input(name, _) => {
                inputs.push(InputCell { cref });
            }
            Cell::Output(name, _) => {
                outputs.push(OutputCell { cref });
            }
            _ => {}
        }
    }

    (inputs, outputs)
}

fn are_inputs_compatible<'p, 'd>(
    pattern_inputs: &[Result<(CellRef<'p>, usize), Trit>],
    design_inputs: &[Result<(CellRef<'d>, usize), Trit>],
    mapping: &HashMap<CellRef<'p>, CellRef<'d>>,
) -> bool {
    if pattern_inputs.len() != design_inputs.len() { return false; }
    for (p_in, d_in) in pattern_inputs.iter().zip(design_inputs.iter()) {
        match (p_in, d_in) {
            (Err(a), Err(b)) => {
                if a != b { return false; }
            }
            (Ok((p_src, p_bit)), Ok((d_src, d_bit))) => {
                if let Some(mapped_d_src) = mapping.get(p_src) {
                    if mapped_d_src != d_src || p_bit != d_bit { return false; }
                }
            }
            _ => return false,
        }
    }
    true
}

// Choose next unmapped pattern cell to extend mapping, preferring those whose inputs are all constants or already mapped sources
fn choose_next_pattern_cell<'p>(
    pattern_cells: &HashMap<CellRef<'p>, (CellKind, Vec<Result<(CellRef<'p>, usize), Trit>>)>,
    mapping: &SubgraphMatch<'p, '_>,
) -> Option<CellRef<'p>> {
    // Prefer cells whose inputs are all constants or mapped sources
    for (&cref, (_kind, inputs)) in pattern_cells.iter() {
        if mapping.cell_mapping.contains_key(&cref) { continue; }
        let mut all_sources_mapped = true;
        for inp in inputs {
            if let Ok((src, _)) = inp {
                if !mapping.cell_mapping.contains_key(src) {
                    all_sources_mapped = false;
                    break;
                }
            }
        }
        if all_sources_mapped { return Some(cref); }
    }
    // Fallback: any unmapped pattern cell
    for (&cref, _) in pattern_cells.iter() {
        if !mapping.cell_mapping.contains_key(&cref) { return Some(cref); }
    }
    None
}

fn backtrack_mappings<'p, 'd>(
    pattern_cells: &HashMap<CellRef<'p>, (CellKind, Vec<Result<(CellRef<'p>, usize), Trit>>)>,
    design_cells: &HashMap<CellRef<'d>, (CellKind, Vec<Result<(CellRef<'d>, usize), Trit>>)>,
    design_cells_by_kind: &HashMap<CellKind, Vec<CellRef<'d>>>,
    mapping: &mut SubgraphMatch<'p, 'd>,
    used_design_cells: &mut HashSet<CellRef<'d>>,
    mappings_out: &mut Vec<SubgraphMatch<'p, 'd>>,
) {
    if mapping.cell_mapping.len() == pattern_cells.len() {
        mappings_out.push(mapping.clone());
        return;
    }

    let Some(next_pattern_cref) = choose_next_pattern_cell(pattern_cells, mapping) else { return; };
    let (pattern_kind, pattern_inputs) = &pattern_cells[&next_pattern_cref];

    let Some(design_candidate_crefs) = design_cells_by_kind.get(pattern_kind) else { return; };

    for &design_cref in design_candidate_crefs.iter() {
        if used_design_cells.contains(&design_cref) { continue; }
        let (_d_kind, design_inputs) = &design_cells[&design_cref];
        if !are_inputs_compatible(pattern_inputs, design_inputs, &mapping.cell_mapping) { continue; }

        mapping.cell_mapping.insert(next_pattern_cref, design_cref);
        used_design_cells.insert(design_cref);

        backtrack_mappings(
            pattern_cells,
            design_cells,
            design_cells_by_kind,
            mapping,
            used_design_cells,
            mappings_out,
        );

        used_design_cells.remove(&design_cref);
        mapping.cell_mapping.remove(&next_pattern_cref);
    }
}

pub fn find_subgraphs<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
) -> Vec<SubgraphMatch<'p, 'd>> {
    let pattern_cell_types = count_cells_by_kind(pattern)
    .into_iter()
    .filter(|(kind, _)| is_matchable_kind(*kind))
    .collect::<Vec<_>>();
let design_cell_types = count_cells_by_kind(design)
    .into_iter()
    .filter(|(kind, _)| is_matchable_kind(*kind))
    .collect::<Vec<_>>();

    // find the smallest cell kind in the design that is also in the pattern
    let anchor_kind = pattern_cell_types
        .iter()
        .filter_map(|(kind, _)| {
            design_cell_types
                .iter()
                .find(|(d_kind, _)| d_kind == kind)
                .map(|_| *kind)
        })
        .min_by_key(|kind| {
            design_cell_types
                .iter()
                .find(|(d_kind, _)| d_kind == kind)
                .map(|(_, count)| *count)
        })
        .expect("No common cell kind found between pattern and design");

    // Build gate-only cell maps and buckets keyed by CellRef
    let (pattern_cells_map, _pattern_by_kind_unused) = collect_matchable_cells(pattern);
    let (design_cells_map, design_cells_by_kind) = collect_matchable_cells(design);

    // Extract anchor CellRefs
    let pattern_anchor_crefs: Vec<CellRef<'p>> = pattern_cells_map
        .iter()
        .filter(|(_, (kind, _))| *kind == anchor_kind)
        .map(|(&cref, _)| cref)
        .collect();

    let design_anchor_crefs: Vec<CellRef<'d>> =
        design_cells_by_kind.get(&anchor_kind).cloned().unwrap_or_default();

    let mut mappings: Vec<SubgraphMatch<'p, 'd>> = Vec::new();

    let Some(pattern_anchor_cref) = pattern_anchor_crefs.first().copied() else {
        log::warn!("Pattern has no anchor cells of kind {:?}", anchor_kind);
        return mappings; // No anchors means no matches
    };

    for &design_anchor_cref in &design_anchor_crefs {
        let (pat_input_cells_map, pat_output_cells_map) = get_pattern_io_cells(pattern);
        let mut mapping = SubgraphMatch {
            cell_mapping: HashMap::from([(pattern_anchor_cref, design_anchor_cref)]),
            pat_input_cells: pat_input_cells_map.clone(),
            pat_output_cells: pat_output_cells_map.clone(),
        };
        let mut used_design: HashSet<CellRef<'d>> = HashSet::new();
        used_design.insert(design_anchor_cref);

        backtrack_mappings(
            &pattern_cells_map,
            &design_cells_map,
            &design_cells_by_kind,
            &mut mapping,
            &mut used_design,
            &mut mappings,
        );
    }

    mappings
}

#[cfg(test)]
mod tests {
    use crate::{get_name, read_input_to_design};

    use super::*;

    #[test]
    fn test_many_regs() {
        let haystack_path = "examples/patterns/security/access_control/locked_reg/json/many_locked_regs.json";
        let haystack_design = read_input_to_design(None, haystack_path.to_string()).expect("Failed to read input design");
        let haystack_name = get_name(&haystack_path);

        let needle_path = "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
        let needle_design = read_input_to_design(None, needle_path.to_string()).expect("Failed to read input design");
        let needle_name = get_name(&needle_path);

        // Find subgraphs using the chosen anchor kind
        let matches = find_subgraphs(&needle_design, &haystack_design);
        assert_eq!(matches.len(), 2, "Expected exactly two matches for {} with {}", needle_name, haystack_name);
    }
}