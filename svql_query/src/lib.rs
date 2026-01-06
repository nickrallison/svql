//! Core query definitions and structures for SVQL.
//!
//! This module defines the fundamental types used to represent query states,
//! hardware wires, and the relationships between pattern components and
//! matched design elements.

extern crate self as svql_query;

pub mod binding;
pub mod composites;
pub mod instance;
// pub mod ir;
pub mod prelude;
pub mod primitives;
pub mod report;
pub mod security;
pub mod traits;
pub mod variants;

/// Re-export of driver crate with Context flattened for macro compatibility
pub mod driver {
    pub use svql_driver::context::Context;
    pub use svql_driver::*;
}

pub use svql_common as common;
pub use svql_subgraph as subgraph;

use prelude::*;

/// A high-level helper to execute a query by type.
///
/// This handles:
/// 1. Loading the design (if not loaded).
/// 2. Building the specific Context for this query.
/// 3. Instantiating the query root.
/// 4. Running the search.
pub fn execute_query<P>(
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
) -> Result<Vec<P::Match>, Box<dyn std::error::Error>>
where
    P: traits::Pattern + 'static,
{
    // 1. Build the Context
    let needle_ctx = P::context(driver, &config.needle_options)?;

    // 2. Add the Haystack to the context
    // We assume the driver already has the design loaded at `key`
    let design_container = driver
        .get_design(key)
        .ok_or_else(|| format!("Design not found in driver: {:?}", key))?;

    let context = needle_ctx.with_design(key.clone(), design_container);

    // 3. Instantiate the Query Root
    let root_name = std::any::type_name::<P>()
        .split("::")
        .last()
        .unwrap_or("query")
        .to_lowercase();
    let query_instance = P::instantiate(Instance::root(root_name));

    // 4. Execute
    let results = query_instance.execute(driver, &context, key, config);

    Ok(results)
}

/// Defines the state of a query component.
///
/// Components exist in two primary states:
/// 1. `Search`: The initial state where components define the pattern to find.
/// 2. `Match`: The result state where components hold references to design elements.
pub trait State: Clone + std::fmt::Debug + PartialEq {
    /// The internal data type held by a Wire in this state.
    type WireInner: Clone + std::fmt::Debug + PartialEq;

    /// Helper to extract source location from the inner wire type.
    fn wire_source(inner: &Self::WireInner) -> Option<SourceLocation> {
        let _ = inner;
        None
    }
}

/// Represents a query in its search/definition phase.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Search;

impl State for Search {
    type WireInner = ();
}

/// Represents a query result bound to specific design elements.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Match {
    /// Owned metadata about the matched cell in the target design.
    pub cell: Option<subgraph::cell::CellInfo>,
}

impl Match {
    /// Retrieves the source code location of the matched design element.
    pub fn source(&self) -> Option<SourceLocation> {
        self.cell.as_ref().and_then(|cell| cell.get_source())
    }
}

impl State for Match {
    type WireInner = Option<CellInfo>;

    fn wire_source(inner: &Self::WireInner) -> Option<SourceLocation> {
        inner.as_ref().and_then(|cell| cell.get_source())
    }
}

/// A logical connection point within a query component.
#[derive(Clone, Debug, PartialEq)]
pub struct Wire<S: State>
where
    S::WireInner: Clone + std::fmt::Debug + PartialEq,
{
    /// Hierarchical path of the wire.
    pub path: Instance,
    /// State-specific data (empty for Search, CellWrapper for Match).
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

impl<'ctx> Wire<Match> {
    /// Returns the matched cell associated with this wire.
    pub fn cell(&self) -> Option<&subgraph::cell::CellInfo> {
        self.inner.as_ref()
    }
}

use crate::traits::{Hardware, Matched, Pattern};

impl<S: State> Hardware for Wire<S> {
    type State = S;

    fn path(&self) -> &Instance {
        &self.path
    }
    fn type_name(&self) -> &'static str {
        "Wire"
    }
    fn children(&self) -> Vec<&dyn Hardware<State = Self::State>> {
        Vec::new()
    }

    fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
        if path == &self.path { Some(self) } else { None }
    }

    fn source(&self) -> Option<SourceLocation> {
        S::wire_source(&self.inner)
    }
}

impl Pattern for Wire<Search> {
    type Match = Wire<Match>;

    fn instantiate(base_path: Instance) -> Self {
        Wire::new(base_path, ())
    }

    fn context(
        _driver: &Driver,
        _config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        Ok(Context::default())
    }

    fn execute(
        &self,
        _driver: &Driver,
        _context: &Context,
        _key: &DriverKey,
        _config: &Config,
    ) -> Vec<Self::Match> {
        vec![]
    }
}

impl Matched for Wire<Match> {
    type Search = Wire<Search>;
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
