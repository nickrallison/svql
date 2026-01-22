//! Core query definitions and structures for SVQL.
//!
//! This module defines the fundamental types used to represent query states,
//! hardware wires, and the relationships between pattern components and
//! matched design elements.

extern crate self as svql_query;

pub mod binding;
pub mod instance;
pub mod prelude;
pub mod report;
pub mod session;
pub mod traits;

/// Re-export of driver crate with Context flattened for macro compatibility
pub mod driver {
    pub use svql_driver::context::Context;
    pub use svql_driver::*;
}

pub use svql_common as common;
pub use svql_subgraph as subgraph;

use prelude::*;
use traits::Hardware;

/// Execute a query with custom execution configuration.
///
/// Allows specifying parallel vs sequential execution and thread limits.
///
/// # Example
///
/// ```ignore
/// use svql_query::prelude::*;
///
/// // Run in parallel with default thread count
/// let config = ExecConfig::parallel();
/// let store = run_query_with_config::<MyPattern<Search>>(&driver, &key, config)?;
///
/// // Or limit threads
/// let config = ExecConfig { parallel: true, max_threads: Some(4) };
/// let store = run_query_with_config::<MyPattern<Search>>(&driver, &key, config)?;
/// ```
pub fn run_query<P>(
    driver: &Driver,
    key: &DriverKey,
    config: session::Config,
) -> Result<session::Store, Box<dyn std::error::Error>>
where
    P: traits::SearchableComponent + Send + Sync + 'static,
{
    let plan = session::ExecutionPlan::for_pattern::<P>()?;
    let store = plan.execute(driver, key.clone(), config)?;

    Ok(store)
}

/// Defines the state of a query component.
///
/// Components exist in two primary states:
/// 1. `Search`: The initial state where components define the pattern to find.
/// 2. `Match`: The result state where components hold references to design elements.
pub trait State: Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash {
    /// The internal data type held by a Wire in this state.
    type WireInner: Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash;

    /// Helper to extract source location from the inner wire type.
    fn wire_source(inner: &Self::WireInner) -> Option<SourceLocation> {
        let _ = inner;
        None
    }
}

/// Represents a query in its search/definition phase.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Search;

impl State for Search {
    type WireInner = ();
}

/// Represents a query result bound to specific design elements.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Match;

impl State for Match {
    type WireInner = Option<CellInfo>;

    fn wire_source(inner: &Self::WireInner) -> Option<SourceLocation> {
        inner.as_ref().and_then(|cell| cell.get_source())
    }
}

/// A logical connection point within a query component.
///
/// `Wire` is a simple data container, not a searchable pattern.
/// In `Search` state, it holds only a path. In `Match` state,
/// it additionally holds information about the matched design cell.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Wire<S: State>
where
    S::WireInner: Clone + std::fmt::Debug + PartialEq + Eq,
{
    /// Hierarchical path of the wire.
    pub path: Instance,
    /// State-specific data (empty for Search, CellInfo for Match).
    pub inner: S::WireInner,
}

impl<S: State> Wire<S> {
    /// Creates a new wire instance.
    pub fn new(path: Instance, inner: S::WireInner) -> Self {
        Self { path, inner }
    }

    /// Returns the hierarchical path of the wire.
    pub fn path(&self) -> &Instance {
        &self.path
    }
}

impl Wire<Search> {
    /// Creates a search wire at the given path.
    pub fn search(path: Instance) -> Self {
        Self::new(path, ())
    }
}

impl Wire<Match> {
    /// Returns the matched cell associated with this wire.
    pub fn cell(&self) -> Option<&subgraph::cell::CellInfo> {
        self.inner.as_ref()
    }

    /// Creates a match wire with the given cell info.
    pub fn matched(path: Instance, cell: Option<CellInfo>) -> Self {
        Self::new(path, cell)
    }
}

/// Hardware implementation for Wire.
///
/// Wire implements Hardware so it can be returned in `children()` trait object vectors,
/// but it is NOT a searchable component - it's just a data container.
impl<S: State> Hardware for Wire<S> {
    type State = S;

    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "Wire"
    }

    fn children(&self) -> Vec<&dyn Hardware<State = Self::State>> {
        Vec::new() // Wires are leaf nodes
    }

    fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
        if path == &self.path { Some(self) } else { None }
    }

    fn source(&self) -> Option<SourceLocation> {
        S::wire_source(&self.inner)
    }
}

/// Represents a connection between two wires.
#[derive(Debug, Clone, PartialEq)]
pub struct Connection<S>
where
    S: State,
{
    pub from: Wire<S>,
    pub to: Wire<S>,
}
