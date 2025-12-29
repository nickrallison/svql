use prjunnamed_netlist::Design;
use svql_driver::DriverKey;
use svql_subgraph::{AssignmentSet, SingleAssignment, cell::CellWrapper};

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
        tracing::event!(tracing::Level::DEBUG, "Creating driver key for netlist");
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }
}

pub fn resolve_wire<'a>(
    embedding: &SingleAssignment<'a, 'a>,
    embedding_set: &AssignmentSet<'a, 'a>,
    _needle: &Design,
    wire_name: &str,
) -> Option<CellWrapper<'a>> {
    if let Some(drivers) = embedding_set.needle_output_fanin_by_name.get(wire_name) {
        if let Some((needle_cell, _)) = drivers.first() {
            return embedding.get_haystack_cell(needle_cell.clone());
        }
    }

    if let Some(sinks) = embedding_set.needle_input_fanout_by_name.get(wire_name) {
        if let Some((needle_cell, _)) = sinks.first() {
            return embedding.get_haystack_cell(needle_cell.clone());
        }
    }

    None
}
