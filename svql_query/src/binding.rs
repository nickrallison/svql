//! Utilities for binding query ports to design matches.
//!
//! Provides functions to map named ports and bit indices from a pattern
//! to the corresponding cells discovered during subgraph isomorphism.

use crate::{Match, Wire, instance::Instance};
use std::collections::HashMap;
use svql_subgraph::{GraphIndex, SingleAssignment, cell::CellWrapper};

/// Pre-computed map of Port Name -> Representative Needle Cell.
/// Created once per query execution, not per match.
pub struct PortResolver<'a> {
    /// Maps "clk" -> The specific cell in the needle graph that represents this input.
    needle_anchors: HashMap<String, CellWrapper<'a>>,
}

impl<'a> PortResolver<'a> {
    /// Analyzes the needle graph to find the best anchor cell for each port.
    pub fn new(index: &GraphIndex<'a>) -> Self {
        let mut needle_anchors = HashMap::new();

        // 1. Map Inputs
        // We pick the first stable cell connected to the input as the anchor.
        for (name, fanout) in index.get_input_fanout_by_name() {
            if let Some((cell, _)) = fanout.first() {
                needle_anchors.insert(name, cell.clone());
            }
        }

        // 2. Map Outputs
        // Similarly for outputs, we pick the driving cell.
        for (name, fanin) in index.get_output_fanin_by_name() {
            if let Some((cell, _)) = fanin.first() {
                needle_anchors.insert(name, cell.clone());
            }
        }

        Self { needle_anchors }
    }

    /// Resolves a port name to a Match using the current assignment.
    pub fn resolve(
        &self,
        port_name: &str,
        assignment: &SingleAssignment<'a, 'a>,
    ) -> Option<crate::subgraph::cell::CellInfo> {
        let needle_cell = self.needle_anchors.get(port_name)?;
        let design_cell = assignment.get_haystack_cell(needle_cell.clone())?;

        Some(design_cell.to_info())
    }

    /// Helper for macros to construct a Wire in the Match state.
    pub fn bind_wire(
        &self,
        path: Instance,
        assignment: &SingleAssignment<'a, 'a>,
        wire_name: &str,
    ) -> Wire<Match> {
        let cell_info = self.resolve(wire_name, assignment);
        Wire::new(path, cell_info)
    }
}
