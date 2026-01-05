use crate::prelude::*;
use svql_subgraph::SubgraphMatcher;
use tracing::debug;

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum PortDir {
//     In,
//     Out,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct PortSpec {
//     pub name: &'static str,
//     pub dir: PortDir,
// }

pub trait Netlist {
    const MODULE_NAME: &'static str;
    const FILE_PATH: &'static str;
    // const PORTS: &'static [PortSpec];

    fn driver_key() -> DriverKey {
        debug!(
            "Creating driver key for netlist: {}, file: {}",
            Self::MODULE_NAME,
            Self::FILE_PATH
        );
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }
}

/// Executes the subgraph isomorphism search for a netlist component.
pub fn execute_netlist_query<'ctx, Q>(
    _query: &Q,
    context: &'ctx Context,
    key: &DriverKey,
    config: &Config,
) -> AssignmentSet<'ctx, 'ctx>
where
    Q: Netlist,
{
    let needle_key = Q::driver_key();
    let needle_container = context
        .get(&needle_key)
        .expect("Pattern design not found in context")
        .as_ref();

    let haystack_container = context
        .get(key)
        .expect("Haystack design not found in context")
        .as_ref();

    SubgraphMatcher::enumerate_with_indices(
        needle_container.design(),
        haystack_container.design(),
        needle_container.index(),
        haystack_container.index(),
        needle_key.module_name().to_string(),
        key.module_name().to_string(),
        config,
    )
}

/// Generates a report node by aggregating source information from all ports.
pub fn report_netlist(
    path: &Instance,
    type_name: &'static str,
    wires: &[&Wire<Match>],
) -> ReportNode {
    let mut all_lines = Vec::new();
    let mut file_path = std::sync::Arc::from("");
    let mut seen = std::collections::HashSet::new();

    for wire in wires {
        if let Some(loc) = wire.inner.as_ref().and_then(|c| c.get_source()) {
            file_path = loc.file;
            for line in loc.lines {
                if seen.insert(line.number) {
                    all_lines.push(line);
                }
            }
        }
    }

    all_lines.sort_by_key(|l| l.number);

    ReportNode {
        name: String::new(),
        type_name: type_name.to_string(),
        path: path.clone(),
        details: None,
        source_loc: if file_path.is_empty() {
            None
        } else {
            Some(SourceLocation {
                file: file_path,
                lines: all_lines,
            })
        },
        children: Vec::new(),
    }
}
