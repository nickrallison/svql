// use svql_driver::subgraph::{
//     SubgraphMatch,
//     cell_kind::{CellWrapper, InputCell, OutputCell},
// };

// /// Locate a named pattern output cell in a subgraph match.
// fn find_pat_output<'p, 'd>(
//     m: &'_ SubgraphMatch<'p, 'd>,
//     name: &str,
// ) -> Option<&'p OutputCell<'p>> {
//     m.pat_output_cells
//         .iter()
//         .find(|o| o.name() == Some(name))
// }

// /// Locate a named pattern input cell in a subgraph match.
// fn find_pat_input<'p, 'd>(
//     m: &'_ SubgraphMatch<'p, 'd>,
//     name: &str,
// ) -> Option<&'p InputCell<'p>> {
//     m.pat_input_cells
//         .iter()
//         .find(|i| i.name() == Some(name))
// }

// /// Resolve the design gate driving a given named output of the pattern match.
// /// Returns the matched design Cell (gate) if it exists.
// pub fn design_driver_of_output<'p, 'd>(
//     m: &'_ SubgraphMatch<'p, 'd>,
//     out_name: &str,
// ) -> Option<CellWrapper<'d>> {
//     let pat_out = find_pat_output(m, out_name)?;
//     // In the pattern, the Output is driven by exactly one gate (for simple cases).
//     let pat_src_gate = pat_out.get_gate();
//     let pat_src_gate = CellWrapper::from(pat_src_gate);

//     // Map the pattern gate to the matched design gate.
//     m.cell_mapping.get(&pat_src_gate).copied()
// }

// /// Resolve the design source(s) that feed a given named input of the pattern.
// /// Uses the boundary_src_map that binds pattern IO pins to design sources.
// /// Returns a list of (design cell, bit) pairs.
// pub fn design_sources_of_input<'p, 'd>(
//     m: &'_ SubgraphMatch<'p, 'd>,
//     in_name: &str,
// ) -> Vec<(CellWrapper<'d>, usize)> {
//     let Some(pat_in) = find_pat_input(m, in_name) else {
//         return Vec::new();
//     };
//     m.boundary_src_map
//         .iter()
//         .filter_map(|(&(p_cell, p_bit), &(d_cell, d_bit))| {
//             if p_cell == pat_in.cref {
//                 Some((d_cell, d_bit))
//             } else {
//                 None
//             }
//         })
//         .collect()
// }

// /// Check if the named output of `a` drives the named input of `b`.
// /// This returns true if any bit driver equals any bit source on the other side.
// /// For most 1-bit cases, this is sufficient. For wide buses, you can
// /// post-filter by bit positions if needed.
// pub fn output_connected_to_input<'p, 'd>(
//     a: &'_ SubgraphMatch<'p, 'd>,
//     a_out_name: &str,
//     b: &'_ SubgraphMatch<'p, 'd>,
//     b_in_name: &str,
// ) -> bool {
//     let Some(a_driver) = design_driver_of_output(a, a_out_name) else {
//         return false;
//     };
//     let b_sources = design_sources_of_input(b, b_in_name);
//     b_sources.iter().any(|(cell, _bit)| *cell == a_driver)
// }

// /// Given two sets of matches, produce the cartesian product and filter it by a connectivity rule:
// /// “a’s output `a_out_name` is connected to b’s input `b_in_name`”.
// ///
// /// Returns references to the original matches to let the consumer use them as needed.
// pub fn filter_connected_pairs<'p, 'd>(
//     a_matches: &'_ [SubgraphMatch<'p, 'd>],
//     a_out_name: &str,
//     b_matches: &'_ [SubgraphMatch<'p, 'd>],
//     b_in_name: &str,
// ) -> Vec<(&'_. SubgraphMatch<'p, 'd>, &'_. SubgraphMatch<'p, 'd>)> {
//     let mut out = Vec::new();
//     for a in a_matches {
//         for b in b_matches {
//             if output_connected_to_input(a, a_out_name, b, b_in_name) {
//                 out.push((a, b));
//             }
//         }
//     }
//     out
// }