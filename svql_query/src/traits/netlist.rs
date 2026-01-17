//! Netlist component traits and utilities.
//!
//! Provides traits for components defined via external HDL files.

use crate::prelude::*;
use crate::traits::component::{MatchedComponent, SearchableComponent, kind};
use svql_subgraph::SubgraphMatcher;
use tracing::debug;

/// Trait for netlist-based pattern components.
///
/// Implemented by types generated with `#[netlist]`. Provides access to
/// the source file path and module name.
pub trait NetlistComponent: SearchableComponent<Kind = kind::Netlist> {
    /// The module name within the source file.
    const MODULE_NAME: &'static str;

    /// Path to the netlist source file (.v, .il, or .json).
    const FILE_PATH: &'static str;

    /// Returns the driver key for this netlist.
    fn driver_key() -> DriverKey {
        debug!(
            "Creating driver key for netlist: {}, file: {}",
            Self::MODULE_NAME,
            Self::FILE_PATH
        );
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }

    /// Binds a subgraph assignment to produce a matched instance.
    fn bind_match(&self, resolver: &PortResolver, assignment: &SingleAssignment) -> Self::Match;
}

/// Trait for the matched state of netlist components.
pub trait NetlistMatched: MatchedComponent {
    type SearchType: NetlistComponent<Match = Self>;
}

/// Executes the subgraph isomorphism search for a netlist component.
pub fn execute_netlist_query<'ctx, Q>(
    _query: &Q,
    context: &'ctx Context,
    key: &DriverKey,
    config: &Config,
) -> AssignmentSet<'ctx, 'ctx>
where
    Q: NetlistComponent,
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
