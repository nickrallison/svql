//! Helper functions for binding query ports to matches.
//!
//! This module provides functions to bind input and output ports of a query
//! to the corresponding cells in the matched design.

use std::collections::HashMap;

use crate::Match;
use svql_subgraph::{SingleAssignment, cell::CellWrapper};
use tracing::debug;

/// Binds a named input port bit to a match.
pub fn bind_input<'ctx>(
    m: &SingleAssignment<'ctx, 'ctx>,
    name: &str,
    bit: usize,
    input_fanout_by_name: &HashMap<String, Vec<(CellWrapper<'ctx>, usize)>>,
) -> Match<'ctx> {
    let pat = input_fanout_by_name.get(name).cloned().unwrap();
    let pat_first = &pat[0].0;
    let des = m.get_haystack_cell(pat_first.clone()).unwrap();
    debug!(
        "bind_input {} bit {} to des {:?} pat: {:?}",
        name, bit, des, pat
    );
    Match {
        pat_node_ref: Some(pat_first.clone()),
        design_node_ref: Some(des),
    }
}

/// Binds a named output port bit to a match.
pub fn bind_output<'ctx>(
    m: &SingleAssignment<'ctx, 'ctx>,
    name: &str,
    bit: usize,
    output_fanin_by_name: &HashMap<String, Vec<(CellWrapper<'ctx>, usize)>>,
) -> Match<'ctx> {
    let pat = output_fanin_by_name.get(name).cloned().unwrap();
    let pat_first = &pat[0].0;
    let des = m.get_haystack_cell(pat_first.clone()).unwrap();
    debug!(
        "bind_output {} bit {} to des {:?} pat: {:?}",
        name, bit, des, pat
    );
    Match {
        pat_node_ref: Some(pat_first.clone()),
        design_node_ref: Some(des),
    }
}
