use crate::{State, Wire};

/// Implemented by Composites to define internal connectivity.
pub trait Topology<S: State> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>);
}

pub struct ConnectionBuilder<'a, S: State> {
    // Outer Vec = AND (All groups must be satisfied)
    // Inner Vec = OR  (At least one pair in the group must connect)
    // Option::None = The port does not exist on the current variant
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
