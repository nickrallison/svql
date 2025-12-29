//! Core query definitions and structures for SVQL.
//!
//! This module defines the fundamental types used to represent query states,
//! hardware wires, and the relationships between pattern components and
//! matched design elements.

extern crate self as svql_query;

use std::sync::Arc;
use svql_subgraph::cell::{CellWrapper, SourceLocation};

pub mod binding;
pub mod composites;
pub mod instance;
pub mod ir;
pub mod primitives;
pub mod report;
pub mod security;
pub mod traits;
pub mod variants;

pub use instance::Instance;
pub use itertools;
pub use svql_common;
pub use svql_driver;
pub use svql_subgraph;
pub use tracing;

use crate::traits::Component;

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
pub struct Match<'ctx> {
    /// Reference to the cell in the pattern/needle design.
    pub pat_node_ref: Option<CellWrapper<'ctx>>,
    /// Reference to the cell in the target/haystack design.
    pub design_node_ref: Option<CellWrapper<'ctx>>,
}

impl<'ctx> Match<'ctx> {
    /// Retrieves the source code location of the matched design element.
    pub fn source(&self) -> Option<SourceLocation> {
        self.design_node_ref.as_ref().and_then(|cell| cell.get_source())
    }
}

impl<'ctx> State for Match<'ctx> {
    type WireInner = CellWrapper<'ctx>;
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

impl<'ctx> Wire<Match<'ctx>> {
    /// Returns the matched cell associated with this wire.
    pub fn cell(&self) -> &CellWrapper<'ctx> {
        &self.inner
    }
}

impl<'ctx> crate::traits::Reportable for Wire<Match<'ctx>> {
    /// Generates a report node for the wire.
    fn to_report(&self, name: &str) -> crate::report::ReportNode {
        crate::report::ReportNode {
            name: name.to_string(),
            type_name: "Wire".to_string(),
            path: self.path.clone(),
            details: None,
            source_loc: self.inner.get_source().unwrap_or_else(|| {
                svql_subgraph::cell::SourceLocation {
                    file: std::sync::Arc::from(""),
                    lines: Vec::new(),
                }
            }),
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
