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

use prelude::*;
use std::sync::Arc;

/// A high-level helper to execute a query by type.
///
/// This handles:
/// 1. Loading the design (if not loaded).
/// 2. Building the specific Context for this query.
/// 3. Instantiating the query root.
/// 4. Running the search.
pub fn execute_query<'a, Q>(
    driver: &Driver,
    key: &DriverKey,
    config: &Config,
) -> Result<Vec<Q::Result<'a>>, Box<dyn std::error::Error>>
where
    // Q represents the <Search> version (e.g., Cwe1234<Search>)
    Q: Projected + Query + Searchable + 'static,
    // We ensure Q::Pattern is actually Q (sanity check)
    Q: Projected<Pattern = Q>,
{
    // 1. Build the Context
    // We use Q::context because Q implements Query
    let needle_ctx = Q::context(driver, &config.needle_options)?;

    // 2. Add the Haystack to the context
    // We assume the driver already has the design loaded at `key`
    let design_container = driver
        .get_design(key)
        .ok_or_else(|| format!("Design not found in driver: {:?}", key))?;

    let context = needle_ctx.with_design(key.clone(), design_container);

    // 3. Instantiate the Query Root
    // We use the type name as the root instance name (e.g., "cwe1234")
    let root_name = std::any::type_name::<Q>()
        .split("::")
        .last()
        .unwrap_or("query")
        .to_lowercase();
    let query_instance = Q::instantiate(Instance::root(root_name));

    // 4. Execute
    let results = query_instance.query(driver, &context, key, config);

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

impl crate::traits::Reportable for Wire<Match> {
    fn to_report(&self, name: &str) -> crate::report::ReportNode {
        crate::report::ReportNode {
            name: name.to_string(),
            type_name: "Wire".to_string(),
            path: self.path.clone(),
            details: None,
            // inner is now Option<CellInfo>, so we map through it
            source_loc: self.inner.as_ref().and_then(|cell| cell.get_source()),
            children: Vec::new(),
        }
    }
}

impl<S: State> Component<S> for Wire<S> {
    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "Wire"
    }

    fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
        if path.starts_with(&self.path) {
            Some(self)
        } else {
            None
        }
    }

    fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        if rel_path.is_empty() {
            Some(self)
        } else {
            None
        }
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
