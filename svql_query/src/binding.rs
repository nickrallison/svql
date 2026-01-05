//! Utilities for binding query ports to design matches.
//!
//! Provides functions to map named ports and bit indices from a pattern
//! to the corresponding cells discovered during subgraph isomorphism.

use crate::Match;
use std::collections::HashMap;
use subgraph::{SingleAssignment, cell::CellWrapper};
use tracing::debug;

/// Binds a named input port bit from a pattern to a design match.
///
/// # Arguments
/// * `assignment` - The mapping between pattern cells and design cells.
/// * `name` - The name of the input port.
/// * `bit_index` - The bit index within the port.
/// * `input_fanout` - Map of port names to their fan-out cell references in the pattern.
pub fn bind_input<'ctx>(
    assignment: &SingleAssignment<'ctx, 'ctx>,
    name: &str,
    bit_index: usize,
    input_fanout: &HashMap<String, Vec<(CellWrapper<'ctx>, usize)>>,
) -> Match {
    let pattern_cells = input_fanout
        .get(name)
        .expect("Input port not found in pattern");
    let pattern_cell = &pattern_cells[bit_index].0;
    let design_cell = assignment
        .get_haystack_cell(pattern_cell.clone())
        .expect("Pattern cell not found in assignment");

    debug!(
        "Binding input {} bit {} to design cell {:?} (pattern: {:?})",
        name, bit_index, design_cell, pattern_cell
    );

    Match {
        cell: Some(design_cell.to_info()),
    }
}

/// Binds a named output port bit from a pattern to a design match.
///
/// # Arguments
/// * `assignment` - The mapping between pattern cells and design cells.
/// * `name` - The name of the output port.
/// * `bit_index` - The bit index within the port.
/// * `output_fanin` - Map of port names to their fan-in cell references in the pattern.
pub fn bind_output<'ctx>(
    assignment: &SingleAssignment<'ctx, 'ctx>,
    name: &str,
    bit_index: usize,
    output_fanin: &HashMap<String, Vec<(CellWrapper<'ctx>, usize)>>,
) -> Match {
    let pattern_cells = output_fanin
        .get(name)
        .expect("Output port not found in pattern");
    let pattern_cell = &pattern_cells[bit_index].0;
    let design_cell = assignment
        .get_haystack_cell(pattern_cell.clone())
        .expect("Pattern cell not found in assignment");

    debug!(
        "Binding output {} bit {} to design cell {:?} (pattern: {:?})",
        name, bit_index, design_cell, pattern_cell
    );

    Match {
        cell: Some(design_cell.to_info()),
    }
}
