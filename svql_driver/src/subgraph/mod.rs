use std::collections::{HashMap, HashSet};

use prjunnamed_netlist::{Cell, CellRef, Design, Trit};

use cell_kind::*;

pub mod cell_kind;

#[derive(Clone, Default)]
pub struct SubgraphMatch<'p, 'd> {
    pub cell_mapping: HashMap<CellRef<'p>, CellRef<'d>>,
    pub pat_input_cells: Vec<InputCell<'p>>,
    pub pat_output_cells: Vec<OutputCell<'p>>,
    pub input_bindings: HashMap<(CellRef<'p>, usize), Result<(CellRef<'d>, usize), Trit>>,
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
        let mut mapping: Vec<((usize, Cell), (usize, Cell))> = self
            .cell_mapping
            .iter()
            .map(|(p, d)| ((p.debug_index(), p.get().as_ref().clone()), (d.debug_index(), d.get().as_ref().clone())))
            .collect();
        mapping.sort_by(|a, b| a.0.0.cmp(&b.0.0));

        let inputs: Vec<InputCell> = self.pat_input_cells.clone();
        let outputs: Vec<OutputCell> = self.pat_output_cells.clone();

        // Render a stable snapshot of input_bindings for debug
        let mut ib: Vec<(usize, usize, Result<(usize, usize), Trit>)> = self
            .input_bindings
            .iter()
            .map(|((p, pb), v)| {
                let pv = p.debug_index();
                let vv = match v {
                    Ok((d, db)) => Ok((d.debug_index(), *db)),
                    Err(t) => Err(*t),
                };
                (pv, *pb, vv)
            })
            .collect();
        ib.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        f.debug_struct("SubgraphMatch")
            .field("cell_mapping", &mapping)
            .field("pat_input_cells", &inputs)
            .field("pat_output_cells", &outputs)
            .field("input_bindings", &ib)
            .finish()
    }
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
        let kind = CellKind::from(cell.get().as_ref());
        if !(kind.is_gate()) {
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
            Cell::Input(_, _) => {
                inputs.push(InputCell { cref });
            }
            Cell::Output(_, _) => {
                outputs.push(OutputCell { cref });
            }
            _ => {}
        }
    }

    (inputs, outputs)
}

fn try_bind_inputs<'p, 'd>(
    pattern_inputs: &[Result<(CellRef<'p>, usize), Trit>],
    design_inputs: &[Result<(CellRef<'d>, usize), Trit>],
    pattern_cells: &HashMap<CellRef<'p>, (CellKind, Vec<Result<(CellRef<'p>, usize), Trit>>)>,
    mapping: &SubgraphMatch<'p, 'd>,
) -> Option<Vec<((CellRef<'p>, usize), Result<(CellRef<'d>, usize), Trit>)>> {
    if pattern_inputs.len() != design_inputs.len() { return None; }

    // local tentative bindings for this step to enforce intra-cell equality too
    let mut tentative: HashMap<(CellRef<'p>, usize), Result<(CellRef<'d>, usize), Trit>> = HashMap::new();

    for (p_in, d_in) in pattern_inputs.iter().zip(design_inputs.iter()) {
        match (p_in, d_in) {
            (Err(a), Err(b)) => {
                if a != b { return None; }
            }
            (Ok((p_src, p_bit)), Ok((d_src, d_bit))) => {
                // If p_src is a gate in the pattern, it must already be mapped to d_src
                if pattern_cells.contains_key(p_src) {
                    match mapping.cell_mapping.get(p_src) {
                        Some(mapped_d_src) => {
                            if mapped_d_src != d_src || p_bit != d_bit { return None; }
                        }
                        None => return None, // gate predecessor not yet mapped
                    }
                } else {
                    // p_src is a boundary (e.g., Input). Enforce equality across uses:
                    let key = (*p_src, *p_bit);
                    if let Some(bound) = mapping.input_bindings.get(&key) {
                        if bound != d_in { return None; }
                    } else if let Some(prev) = tentative.get(&key) {
                        if prev != d_in { return None; }
                    } else {
                        tentative.insert(key, Ok((*d_src, *d_bit)));
                    }
                }
            }
            (Ok((p_src, p_bit)), Err(b)) => {
                // boundary pattern source mapping to a constant in the design
                if pattern_cells.contains_key(p_src) {
                    return None;
                }
                let key = (*p_src, *p_bit);
                if let Some(bound) = mapping.input_bindings.get(&key) {
                    if bound != d_in { return None; }
                } else if let Some(prev) = tentative.get(&key) {
                    if prev != d_in { return None; }
                } else {
                    tentative.insert(key, Err(*b));
                }
            }
            _ => return None,
        }
    }

    Some(tentative.into_iter().collect())
}

fn check_forward_consistency<'p, 'd>(
    newly_mapped_p: CellRef<'p>,
    newly_mapped_d: CellRef<'d>,
    pattern_cells: &HashMap<CellRef<'p>, (CellKind, Vec<Result<(CellRef<'p>, usize), Trit>>)>,
    design_cells: &HashMap<CellRef<'d>, (CellKind, Vec<Result<(CellRef<'d>, usize), Trit>>)>,
    mapping: &SubgraphMatch<'p, 'd>,
) -> bool {
    for (p_dst, d_dst) in mapping.cell_mapping.iter() {
        let (_, p_inputs) = &pattern_cells[p_dst];
        let (_, d_inputs) = &design_cells[d_dst];

        // Inputs vectors must be same length by construction of both cell kinds
        for (i, p_in) in p_inputs.iter().enumerate() {
            if let Ok((p_src, p_bit)) = p_in {
                if *p_src == newly_mapped_p {
                    match d_inputs.get(i) {
                        Some(Ok((d_src, d_bit))) => {
                            // Must be driven by the newly mapped design gate on the same bit
                            if *d_src != newly_mapped_d || *d_bit != *p_bit {
                                return false;
                            }
                        }
                        _ => return false,
                    }
                }
            }
        }
    }
    true
}

// Choose next unmapped pattern cell to extend mapping, preferring those whose
// inputs are all constants, primary inputs, or already mapped gate sources.
// Do NOT pick a cell with an unmapped gate predecessor.
fn choose_next_pattern_cell<'p>(
    pattern_cells: &HashMap<CellRef<'p>, (CellKind, Vec<Result<(CellRef<'p>, usize), Trit>>)>,
    mapping: &SubgraphMatch<'p, '_>,
) -> Option<CellRef<'p>> {
    // Prefer cells whose gate-source predecessors are all mapped
    for (&cref, (_kind, inputs)) in pattern_cells.iter() {
        if mapping.cell_mapping.contains_key(&cref) { continue; }
        let mut ok = true;
        for inp in inputs {
            if let Ok((src, _)) = inp {
                // Only gate predecessors matter for ordering
                if pattern_cells.contains_key(src) && !mapping.cell_mapping.contains_key(src) {
                    ok = false;
                    break;
                }
            }
        }
        if ok { return Some(cref); }
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

        // Inputs compatibility (gate predecessors must be mapped, and maintain PI equality)
        let Some(new_pi_binds) = try_bind_inputs(
            pattern_inputs,
            design_inputs,
            pattern_cells,
            mapping,
        ) else { continue; };

        // Tentatively insert to mapping and bindings and check forward-consistency
        mapping.cell_mapping.insert(next_pattern_cref, design_cref);
        let mut added_keys: Vec<(CellRef<'p>, usize)> = Vec::new();
        for (k, v) in new_pi_binds {
            // we only ever add new keys; existing consistency already checked
            mapping.input_bindings.insert(k, v);
            added_keys.push(k);
        }

        if check_forward_consistency(next_pattern_cref, design_cref, pattern_cells, design_cells, mapping) {
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
        }

        // backtrack
        mapping.cell_mapping.remove(&next_pattern_cref);
        for k in added_keys {
            mapping.input_bindings.remove(&k);
        }
    }
}

pub fn find_subgraphs<'p, 'd>(
    pattern: &'p Design,
    design: &'d Design,
) -> Vec<SubgraphMatch<'p, 'd>> {
    let pattern_cell_types = count_cells_by_kind(pattern, is_gate_cell_ref)
        .into_iter()
        .filter(|(kind, _)| kind.is_gate())
        .collect::<Vec<_>>();
    let design_cell_types = count_cells_by_kind(design, is_gate_cell_ref)
        .into_iter()
        .filter(|(kind, _)| kind.is_gate())
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
        });

    let Some(anchor_kind) = anchor_kind else {
        log::warn!("No anchor kind found");
        return Vec::new();
    };

    let (pattern_cells_map, _) = collect_matchable_cells(pattern);
    let (design_cells_map, design_cells_by_kind) = collect_matchable_cells(design);

    // Pattern anchors of the anchor kind
    let mut pattern_anchor_crefs: Vec<CellRef<'p>> = pattern_cells_map
        .iter()
        .filter(|(_, (kind, _))| *kind == anchor_kind)
        .map(|(&cref, _)| cref)
        .collect();

    // Prefer an anchor with no gate predecessors (more deterministic and valid start)
    let mut preferred: Vec<CellRef<'p>> = pattern_anchor_crefs
        .iter()
        .copied()
        .filter(|cref| {
            if let Some((_k, inputs)) = pattern_cells_map.get(cref) {
                for inp in inputs {
                    if let Ok((src, _)) = inp {
                        if pattern_cells_map.contains_key(src) {
                            return false;
                        }
                    }
                }
            }
            true
        })
        .collect();
    if preferred.is_empty() {
        preferred = pattern_anchor_crefs.clone();
    }
    // Pick a single anchor to avoid duplicate isomorphisms
    let Some(pattern_anchor_cref) = preferred.first().copied() else {
        log::warn!("Pattern has no anchor cells of kind {:?}", anchor_kind);
        return Vec::new();
    };

    let design_anchor_crefs: Vec<CellRef<'d>> =
        design_cells_by_kind.get(&anchor_kind).cloned().unwrap_or_default();

    let mut mappings: Vec<SubgraphMatch<'p, 'd>> = Vec::new();

    for &design_anchor_cref in &design_anchor_crefs {
        let (pat_input_cells_map, pat_output_cells_map) = get_pattern_io_cells(pattern);
        let mut mapping = SubgraphMatch {
            cell_mapping: HashMap::from([(pattern_anchor_cref, design_anchor_cref)]),
            pat_input_cells: pat_input_cells_map.clone(),
            pat_output_cells: pat_output_cells_map.clone(),
            input_bindings: HashMap::new(),
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

    // Deduplicate identical mappings (by the set of p->d pairs)
    {
        let mut seen: HashSet<String> = HashSet::new();
        mappings.retain(|m| {
            let mut pairs: Vec<(usize, usize)> = m.cell_mapping
                .iter()
                .map(|(p, d)| (p.debug_index(), d.debug_index()))
                .collect();
            pairs.sort_by_key(|x| x.0);
            let sig = pairs.iter()
                .map(|(a, b)| format!("{}->{}", a, b))
                .collect::<Vec<_>>()
                .join(",");
            seen.insert(sig)
        });
    }

    mappings
}

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::read_input_to_design;
//     use cell_kind::{
//         count_cells_by_kind, get_input_cells, get_output_cells, CellKind,
//     };

//     lazy_static::lazy_static! {
//         static ref ASYNC_MUX: Design = load_design_from("examples/patterns/security/access_control/locked_reg/json/async_mux.json");
//         static ref DOUBLE_SDFFE: Design = load_design_from("examples/patterns/basic/ff/double_sdffe.v");
//         static ref SDFFE: Design = load_design_from("examples/patterns/basic/ff/sdffe.v");
//     }

//     fn load_design_from(path: &str) -> Design {
//         read_input_to_design(None, path.to_string()).expect("Failed to read input design")
//     }


//     // Exercises SubgraphMatch::{len, is_empty, iter} and Debug impl on an empty mapping.
//     #[test]
//     fn test_subgraphmatch_basic_methods_and_debug_empty() {
//         let m: SubgraphMatch = Default::default();
//         assert!(m.is_empty(), "Default SubgraphMatch should be empty");
//         assert_eq!(m.len(), 0, "Default SubgraphMatch length should be 0");
//         assert!(m.iter().next().is_none(), "Iterator over empty mapping should be empty");

//         let dbg = format!("{m:?}");
//         assert!(dbg.contains("SubgraphMatch"), "Debug should include struct name");
//         assert!(dbg.contains("cell_mapping"), "Debug should include cell_mapping");
//         assert!(dbg.contains("pat_input_cells"), "Debug should include pat_input_cells");
//         assert!(dbg.contains("pat_output_cells"), "Debug should include pat_output_cells");
//     }

//     #[test]
//     fn test_get_input_cells() {
//         let mut inputs = get_input_cells(&ASYNC_MUX);
//         inputs.sort();

//         assert_eq!(inputs.len(), 4, "Expected 4 input cells");

//         let actual = inputs;
//         let _actual_clk = actual.iter().find(|cell| cell.name() == Some("clk")).cloned().expect("Expected clk input cell");
//         let _actual_data_in = actual.iter().find(|cell| cell.name() == Some("data_in")).cloned().expect("Expected data_in input cell");
//         let _actual_resetn = actual.iter().find(|cell| cell.name() == Some("resetn")).cloned().expect("Expected resetn input cell");
//         let _actual_write_en = actual.iter().find(|cell| cell.name() == Some("write_en")).cloned().expect("Expected write_en input cell");
//     }

//     #[test]
//     fn test_get_output_cells() {
//         let mut outputs = get_output_cells(&ASYNC_MUX);
//         outputs.sort();

//         assert_eq!(outputs.len(), 1, "Expected 1 output cell");

//         let actual = outputs;
//         let _actual_data_out = actual.iter().find(|cell| cell.name() == Some("data_out")).cloned().expect("Expected data_out output cell");
//     }

//     // Exercises get_pattern_io_cells by comparing against the reference helpers in cell_kind.
//     #[test]
//     fn test_get_pattern_io_cells_collects_io() {
//         let design = &ASYNC_MUX;
//         let (pat_inputs, pat_outputs) = get_pattern_io_cells(&design);

//         // Cross-check lengths with the public helpers from cell_kind.
//         let ref_inputs = get_input_cells(&design);
//         let ref_outputs = get_output_cells(&design);

//         assert_eq!(
//             pat_inputs.len(),
//             ref_inputs.len(),
//             "get_pattern_io_cells should collect the same number of inputs as get_input_cells"
//         );
//         assert_eq!(
//             pat_outputs.len(),
//             ref_outputs.len(),
//             "get_pattern_io_cells should collect the same number of outputs as get_output_cells"
//         );
//     }

//     // Exercises collect_matchable_cells: verifies it collects only gate cells and buckets them by kind.
//     #[test]
//     fn test_collect_matchable_cells_gate_filtering_and_bucketing() {
//         let design = &ASYNC_MUX;

//         let (cell_map, cells_by_kind) = collect_matchable_cells(&design);

//         // Count the number of gate cells in the design using the reference helpers.
//         let gate_count = count_cells_by_kind(&design, is_gate_cell_ref)
//             .into_iter()
//             .map(|(_, c)| c)
//             .sum::<usize>();

//         let input_count = count_cells_by_kind(&design, |c| matches!(c.get().as_ref(), Cell::Input(_, _)))
//             .into_iter()
//             .map(|(_, c)| c)
//             .sum::<usize>();

//         assert_eq!(
//             gate_count + input_count,
//             cell_map.len(),
//             "collect_matchable_cells should collect exactly the gate cells + the input cells"
//         );

//         // Ensure no non-gate cells are present.
//         for cref in design.iter_cells() {
//             let kind = CellKind::from(cref.get().as_ref());
//             let in_map = cell_map.contains_key(&cref);
//             if is_gate_cell_ref(cref) || matches!(kind, CellKind::Input) {
//                 assert!(in_map, "Gate / Input cell {:?} should be in the matchable map", kind);
//             } else if matches!(kind, CellKind::Output) {
//                 assert!(!in_map, "Cells that are not gates / inputs {:?} should NOT be in the matchable map", kind);
//             }
//         }

//         // Check that bucketing by kind is consistent with the collected map.
//         for (kind, crefs) in cells_by_kind {
//             for c in crefs {
//                 let (ck, _inputs) = cell_map.get(&c).expect("Bucketed cell must exist in map");
//                 assert_eq!(
//                     *ck, kind,
//                     "Bucket kind and cell_map kind should agree for the same cell"
//                 );
//             }
//         }
//     }

//     // Exercises are_inputs_compatible for the empty-inputs case (no CellRef construction required).
//     #[test]
//     fn test_are_inputs_compatible_empty_inputs() {
//         let pattern_inputs: Vec<Result<(CellRef, usize), Trit>> = Vec::new();
//         let design_inputs: Vec<Result<(CellRef, usize), Trit>> = Vec::new();
//         let mapping: HashMap<CellRef, CellRef> = HashMap::new();

//         assert!(
//             are_inputs_compatible(&pattern_inputs, &design_inputs, &mapping),
//             "Empty input vectors should be compatible"
//         );
//     }

//     // Exercises choose_next_pattern_cell: smoke test that returns any unmapped pattern cell.
//     #[test]
//     fn test_choose_next_pattern_cell_returns_some_unmapped_cell() {
//         let pattern = &ASYNC_MUX;
//         let (pattern_cells_map, _unused) = collect_matchable_cells(&pattern);

//         // If there are no gate cells, skip the test (or you could assert an expected condition).
//         if pattern_cells_map.is_empty() {
//             eprintln!("Pattern has no gate cells; skipping test.");
//             return;
//         }

//         let mapping: SubgraphMatch = Default::default();
//         let next = choose_next_pattern_cell(&pattern_cells_map, &mapping)
//             .expect("Should return some unmapped cell when any exist");

//         assert!(
//             pattern_cells_map.contains_key(&next),
//             "Returned CellRef must be present in pattern_cells_map"
//         );
//     }

//     // Exercises backtrack_mappings base case: when pattern_cells is empty, it should immediately push the mapping.
//     #[test]
//     fn test_backtrack_mappings_base_case_empty_pattern() {
//         let pattern_cells: HashMap<CellRef, (CellKind, Vec<Result<(CellRef, usize), Trit>>)> =
//             HashMap::new();
//         let design_cells: HashMap<CellRef, (CellKind, Vec<Result<(CellRef, usize), Trit>>)> =
//             HashMap::new();
//         let design_cells_by_kind: HashMap<CellKind, Vec<CellRef>> = HashMap::new();

//         let mut mapping: SubgraphMatch = Default::default();
//         let mut used_design_cells: HashSet<CellRef> = HashSet::new();
//         let mut outputs: Vec<SubgraphMatch> = Vec::new();

//         backtrack_mappings(
//             &pattern_cells,
//             &design_cells,
//             &design_cells_by_kind,
//             &mut mapping,
//             &mut used_design_cells,
//             &mut outputs,
//         );

//         assert_eq!(
//             outputs.len(),
//             1,
//             "Base case with empty pattern should produce one (empty) mapping"
//         );
//         assert!(outputs[0].is_empty(), "Mapping should be empty");
//     }

//     // Exercises Debug impl with a non-empty SubgraphMatch (minimally).
//     #[test]
//     fn test_subgraphmatch_debug_non_empty() {
//         let pattern = &ASYNC_MUX;
//         let design = &ASYNC_MUX;
//         let (p_map, _) = collect_matchable_cells(&pattern);
//         let (d_map, d_by_kind) = collect_matchable_cells(&design);

//         // Find any common kind and pick one cref from each design to map.
//         let mut mapped = SubgraphMatch {
//             cell_mapping: HashMap::new(),
//             ..Default::default()
//         };

//         // Capture IO cells for Debug printing.
//         let (pat_inputs, pat_outputs) = get_pattern_io_cells(&pattern);
//         mapped.pat_input_cells = pat_inputs;
//         mapped.pat_output_cells = pat_outputs;

//         // Try to find at least one gate to create a non-empty mapping.
//         let mut made_one = false;
//         'outer: for (&p_cref, (p_kind, _)) in p_map.iter() {
//             if let Some(d_candidates) = d_by_kind.get(p_kind) {
//                 if let Some(&d_cref) = d_candidates.get(0) {
//                     mapped.cell_mapping.insert(p_cref, d_cref);
//                     made_one = true;
//                     break 'outer;
//                 }
//             }
//         }

//         if !made_one {
//             eprintln!("No common gate kind found to create a non-empty mapping; skipping test.");
//             return;
//         }

//         let dbg = format!("{mapped:?}");
//         assert!(dbg.contains("SubgraphMatch"), "Debug should include struct name");
//         assert!(dbg.contains("cell_mapping"), "Debug should include cell_mapping");
//         assert!(dbg.contains("pat_input_cells"), "Debug should include pat_input_cells");
//         assert!(dbg.contains("pat_output_cells"), "Debug should include pat_output_cells");
//     }

//     // Exercises find_subgraphs on a self-match (pattern == design): should find at least one mapping
//     // and each mapping should cover all gate cells in the pattern.
//     #[test]
//     fn test_find_subgraphs_self_match_smoke() {
//         let pattern = &ASYNC_MUX;
//         let design = &ASYNC_MUX;

//         let (pattern_cells_map, _p_by_kind) = collect_matchable_cells(&pattern);
//         let expected_gate_count = pattern_cells_map.len();

//         // If there are no gates, then the function will .expect(...) on anchor selection.
//         if expected_gate_count == 0 {
//             eprintln!("Design has no gate cells; skipping test.");
//             return;
//         }

//         let matches = find_subgraphs(&pattern, &design);

//         assert!(
//             !matches.is_empty(),
//             "Self-match should produce at least one mapping"
//         );

//         for m in &matches {
//             assert_eq!(
//                 m.len(),
//                 expected_gate_count,
//                 "Each mapping should include all gate cells of the pattern"
//             );
//         }

//         // IO cells should be preserved in matches.
//         let (p_inputs, p_outputs) = get_pattern_io_cells(&pattern);
//         for m in &matches {
//             assert_eq!(
//                 m.pat_input_cells.len(),
//                 p_inputs.len(),
//                 "Match should carry all pattern input cells"
//             );
//             assert_eq!(
//                 m.pat_output_cells.len(),
//                 p_outputs.len(),
//                 "Match should carry all pattern output cells"
//             );
//         }
//     }

//     // Existing test kept as-is (exercises the main end-to-end path and backtracking).
//     #[test]
//     fn test_many_regs() {
//         let haystack_path = "examples/patterns/security/access_control/locked_reg/json/many_locked_regs.json";
//         let haystack_design = read_input_to_design(None, haystack_path.to_string()).expect("Failed to read input design");
//         let haystack_name = crate::get_name(&haystack_path);

//         let needle_path = "examples/patterns/security/access_control/locked_reg/json/async_mux.json";
//         let needle_design = read_input_to_design(None, needle_path.to_string()).expect("Failed to read input design");
//         let needle_name = crate::get_name(&needle_path);

//         // Find subgraphs using the chosen anchor kind
//         let matches = find_subgraphs(&needle_design, &haystack_design);
//         assert_eq!(matches.len(), 2, "Expected exactly two matches for {} with {}", needle_name, haystack_name);
//     }
// }