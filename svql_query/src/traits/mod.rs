//! Core traits for query components and execution.
//!
//! Defines the interfaces that all netlists, composites, and variants must
//! implement to participate in the SVQL query system.

pub mod composite;
pub mod netlist;
pub mod variant;

use crate::prelude::*;

/// The central hardware abstraction.
/// Replaces Component and Projected.
pub trait Hardware: std::fmt::Debug {
    /// The current state (Search or Match).
    type State: State;

    /// Hierarchical path (Moved from Component).
    fn path(&self) -> &Instance;

    /// Static type name (Moved from Component).
    fn type_name(&self) -> &'static str;

    /// Returns a list of immediate child wires/submodules.
    fn children(&self) -> Vec<&dyn Hardware<State = Self::State>>;

    /// Generic implementation of port finding based on children.
    fn find_port(&self, path: &Instance) -> Option<&Wire<Self::State>> {
        if path == self.path() {
            // Wires should override this to return Some(self)
            return None;
        }
        for child in self.children() {
            if path.starts_with(child.path()) {
                if let Some(port) = child.find_port(path) {
                    return Some(port);
                }
            }
        }
        None
    }

    /// Source location retrieval.
    fn source(&self) -> Option<SourceLocation> {
        self.children().iter().find_map(|c| c.source())
    }

    /// Reporting.
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

/// The Pattern trait for Search state.
/// Replaces Searchable and Query.
pub trait Pattern: Hardware<State = Search> + Sized + Clone {
    type Match: Matched<Search = Self>;

    /// Constructor (Moved from Searchable).
    fn instantiate(base_path: Instance) -> Self;

    /// Context setup (Moved from Query).
    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>>;

    /// Execution logic (Moved from Query).
    fn execute(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Match>;
}

/// The Matched trait for Match state.
/// Replaces Reportable.
pub trait Matched: Hardware<State = Match> + Sized + Clone {
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

/// Private helper to resolve CellInfo to CellWrappers and check connectivity.
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
