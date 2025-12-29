//! Core query definitions and structures for SVQL.
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

pub trait State: Clone + std::fmt::Debug + PartialEq {
    type WireInner: Clone + std::fmt::Debug + PartialEq;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Search;
impl State for Search {
    type WireInner = ();
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Match<'ctx> {
    pub pat_node_ref: Option<CellWrapper<'ctx>>,
    pub design_node_ref: Option<CellWrapper<'ctx>>,
}

impl<'ctx> Match<'ctx> {
    pub fn source(&self) -> Option<SourceLocation> {
        if let Some(cell) = &self.design_node_ref {
            cell.get_source()
        } else {
            None
        }
    }
}

impl<'ctx> State for Match<'ctx> {
    type WireInner = CellWrapper<'ctx>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wire<S: State>
where
    S::WireInner: Clone + std::fmt::Debug + PartialEq,
{
    pub path: Instance,
    pub inner: S::WireInner,
}

impl<S: State> Wire<S> {
    pub fn new(path: Instance, inner: S::WireInner) -> Self {
        Self { path, inner }
    }
    pub fn path(&self) -> &Instance {
        &self.path
    }
}

impl<'ctx> Wire<Match<'ctx>> {
    pub fn cell(&self) -> &CellWrapper<'ctx> {
        &self.inner
    }
}

impl<'ctx> crate::traits::Reportable for Wire<Match<'ctx>> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Connection<S>
where
    S: State,
{
    pub from: Wire<S>,
    pub to: Wire<S>,
}
