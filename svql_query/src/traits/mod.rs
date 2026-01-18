//! Core traits for query components and execution.
//!
//! Defines the interfaces that all netlists, composites, and variants must
//! implement to participate in the SVQL query system.

pub mod component;
pub mod composite;
pub mod netlist;
pub mod variant;

// Re-export key traits
pub use component::{MatchedComponent, SearchableComponent, kind};
pub use composite::{
    CompositeComponent, CompositeMatched, ConnectionBuilder, Connections, Topology,
    validate_composite, DehydratedTopologyValidation,
};
pub use netlist::{NetlistComponent, NetlistMatched, execute_netlist_query, report_netlist};
pub use variant::{VariantComponent, VariantMatched};

use crate::prelude::*;

/// The central hardware abstraction for query components.
pub trait Hardware: std::fmt::Debug {
    /// The current state (Search or Match).
    type State: State;

    /// Returns the hierarchical path of the component.
    fn path(&self) -> &Instance;

    /// Returns the static type name of the component.
    fn type_name(&self) -> &'static str;

    /// Returns a list of immediate child wires or submodules.
    fn children(&self) -> Vec<&dyn Hardware<State = Self::State>>;

    /// Performs a recursive lookup for a port wire by its hierarchical path.
    fn find_port(&self, path: &Instance) -> Option<&Wire<Self::State>> {
        if path == self.path() {
            return None;
        }
        for child in self.children() {
            if path.starts_with(child.path())
                && let Some(port) = child.find_port(path) {
                    return Some(port);
                }
        }
        None
    }

    /// Retrieves the source location by aggregating child source information.
    fn source(&self) -> Option<SourceLocation> {
        self.children().iter().find_map(|c| c.source())
    }

    /// Generates a report node for the component and its children.
    fn report(&self, name: &str) -> ReportNode {
        let children_reports = self
            .children()
            .iter()
            .map(|c| c.report(c.path().name()))
            .collect();

        ReportNode {
            name: name.to_string(),
            type_name: self.type_name().to_string(),
            path: self.path().clone(),
            source_loc: self.source(),
            children: children_reports,
            details: None,
        }
    }
}

/// Defines a pattern that can be instantiated and searched within a design.
///
/// **Note:** This trait has a blanket implementation for all types implementing
/// `SearchableComponent`. Manual implementation is only required for custom patterns.
pub trait Pattern: Hardware<State = Search> + Sized + Clone {
    /// The corresponding result type in the Match state.
    type Match: Matched<Search = Self>;

    /// Instantiates the pattern at the specified hierarchical path.
    fn instantiate(base_path: Instance) -> Self;

    /// Prepares the design context required for this pattern.
    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>>;

    /// Executes the pattern search against a design context.
    fn execute(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Match>;
}

/// Represents a query result bound to specific design elements.
///
/// **Note:** This trait has a blanket implementation for all types implementing
/// `MatchedComponent`. Manual implementation is only required for custom patterns.
pub trait Matched: Hardware<State = Match> + Sized + Clone {
    /// The corresponding pattern type in the Search state.
    type Search: Pattern<Match = Self>;
}

/// Validates that a physical connection exists between two matched wires in the haystack.
pub fn validate_connection<'ctx>(
    from: &Wire<Match>,
    to: &Wire<Match>,
    haystack_index: &GraphIndex<'ctx>,
) -> bool {
    validate_connection_inner(from, to, haystack_index).unwrap_or(false)
}

/// Internal helper to resolve CellInfo to CellWrappers and check connectivity.
fn validate_connection_inner<'ctx>(
    from: &Wire<Match>,
    to: &Wire<Match>,
    haystack_index: &GraphIndex<'ctx>,
) -> Option<bool> {
    let from_id = from.inner.as_ref()?.id;
    let to_id = to.inner.as_ref()?.id;

    let f_wrapper = haystack_index.get_cell_by_id(from_id)?;
    let t_wrapper = haystack_index.get_cell_by_id(to_id)?;

    let fanout = haystack_index.fanout_set(&f_wrapper)?;
    Some(fanout.contains(&t_wrapper))
}
