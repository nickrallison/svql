//! Composite component traits and utilities.
//!
//! Provides traits for hierarchical pattern components.

use crate::prelude::*;
use crate::traits::component::{MatchedComponent, SearchableComponent, kind};

/// Trait for composite pattern components.
///
/// Implemented by types generated with `#[composite]`. Composites combine
/// multiple sub-patterns with connectivity constraints.
pub trait CompositeComponent:
    SearchableComponent<Kind = kind::Composite> + Topology<Search>
{
    /// Executes all submodule queries and constructs candidate matches.
    ///
    /// The implementation performs:
    /// 1. Execution of each submodule query
    /// 2. Cartesian product of sub-matches
    /// 3. Filtering via `Topology` connectivity constraints
    fn execute_submodules(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Match>;
}

/// Trait for the matched state of composite components.
pub trait CompositeMatched: MatchedComponent + Topology<Match> {
    type SearchType: CompositeComponent<Match = Self>;
}

/// Implemented by Composites to define internal connectivity.
pub trait Topology<S: State> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>);
}

pub struct ConnectionBuilder<'a, S: State> {
    pub constraints: Vec<Vec<(Option<&'a Wire<S>>, Option<&'a Wire<S>>)>>,
}

impl<'a, S: State> Default for ConnectionBuilder<'a, S> {
    fn default() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }
}

impl<'a, S: State> ConnectionBuilder<'a, S> {
    /// Creates a new empty connection builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a mandatory connection.
    /// If either 'from' or 'to' is None, this constraint evaluates to FALSE.
    pub fn connect<A, B>(&mut self, from: A, to: B)
    where
        A: Into<Option<&'a Wire<S>>>,
        B: Into<Option<&'a Wire<S>>>,
    {
        self.constraints.push(vec![(from.into(), to.into())]);
    }

    /// Adds a flexible connection group (CNF clause).
    /// At least one pair in the list must be valid and connected.
    pub fn connect_any<A, B>(&mut self, options: &[(A, B)])
    where
        A: Into<Option<&'a Wire<S>>> + Clone,
        B: Into<Option<&'a Wire<S>>> + Clone,
    {
        let group = options
            .iter()
            .map(|(a, b)| (a.clone().into(), b.clone().into()))
            .collect();

        self.constraints.push(group);
    }
}

impl From<ConnectionBuilder<'_, Search>> for Connections {
    fn from(builder: ConnectionBuilder<'_, Search>) -> Self {
        Connections {
            constraints: builder
                .constraints
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|(from_opt, to_opt)| {
                            (from_opt.cloned(), to_opt.cloned())
                        })
                        .collect()
                })
                .collect(),
        }
    }
}

/// Validates a candidate composite match against its topology constraints.
pub fn validate_composite<'ctx, T>(candidate: &T, haystack_index: &GraphIndex<'ctx>) -> bool
where
    T: Topology<Match>,
{
    let mut builder = ConnectionBuilder::new();
    candidate.define_connections(&mut builder);

    builder.constraints.iter().all(|group| {
        group
            .iter()
            .any(|(from_opt, to_opt)| match (from_opt, to_opt) {
                (Some(f), Some(t)) => validate_connection(f, t, haystack_index),
                _ => false,
            })
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Connections {
    pub constraints: Vec<Vec<(Option<Wire<Search>>, Option<Wire<Search>>)>>,
}
