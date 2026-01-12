use crate::prelude::*;

/// Implemented by Composites to define internal connectivity.
pub trait Topology<S: State> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>);
}

pub struct ConnectionBuilder<'a, S: State> {
    pub constraints: Vec<Vec<(Option<&'a Wire<S>>, Option<&'a Wire<S>>)>>,
}

impl<'a, S: State> ConnectionBuilder<'a, S> {
    /// Adds a mandatory connection.
    /// If either 'from' or 'to' is None, this constraint evaluates to FALSE.
    pub fn connect<A, B>(&mut self, from: A, to: B)
    where
        A: Into<Option<&'a Wire<S>>>,
        B: Into<Option<&'a Wire<S>>>,
    {
        self.constraints.push(vec![(from.into(), to.into())]);
    }

    /// Adds a flexible connection group (CNF).
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

impl Into<Connections> for ConnectionBuilder<'_, Search> {
    fn into(self) -> Connections {
        Connections {
            constraints: self
                .constraints
                .iter()
                .map(|group| {
                    group
                        .iter()
                        .map(|(from_opt, to_opt)| {
                            (from_opt.map(|w| w.clone()), to_opt.map(|w| w.clone()))
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
    let mut builder = ConnectionBuilder {
        constraints: Vec::new(),
    };
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
