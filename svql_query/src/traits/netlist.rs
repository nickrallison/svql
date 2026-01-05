use prjunnamed_netlist::Design;

use crate::prelude::*;

use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDir {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortSpec {
    pub name: &'static str,
    pub dir: PortDir,
}

pub trait NetlistMeta {
    const MODULE_NAME: &'static str;
    const FILE_PATH: &'static str;
    const PORTS: &'static [PortSpec];

    fn driver_key() -> DriverKey {
        debug!(
            "Creating driver key for netlist: {}, file: {}",
            Self::MODULE_NAME,
            Self::FILE_PATH
        );
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }
}

pub fn resolve_wire<'a>(
    assignment: &SingleAssignment<'a, 'a>,
    assignment_set: &AssignmentSet<'a, 'a>,
    _needle: &Design,
    wire_name: &str,
) -> Option<CellWrapper<'a>> {
    if let Some(drivers) = assignment_set.needle_output_fanin_by_name.get(wire_name) {
        if let Some((needle_cell, _)) = drivers.first() {
            return assignment.get_haystack_cell(needle_cell.clone());
        }
    }

    if let Some(sinks) = assignment_set.needle_input_fanout_by_name.get(wire_name) {
        if let Some((needle_cell, _)) = sinks.first() {
            return assignment.get_haystack_cell(needle_cell.clone());
        }
    }

    None
}

/// Helper for macros to construct a Wire in the Match state.
/// This handles the logic of looking up the cell, converting it to owned info,
/// and handling the Option wrapping.
pub fn bind_match_wire(
    path: Instance,
    assignment: &SingleAssignment,
    assignment_set: &AssignmentSet,
    needle_design: &prjunnamed_netlist::Design,
    wire_name: &str,
) -> Wire<Match> {
    let cell_info =
        resolve_wire(assignment, assignment_set, needle_design, wire_name).map(|cw| cw.to_info()); // Converts borrowed CellWrapper to owned CellInfo

    Wire::new(path, cell_info)
}
