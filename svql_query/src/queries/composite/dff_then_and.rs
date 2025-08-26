// svql_query/src/queries/composite/dff_then_and.rs
use svql_common::Config;
use svql_driver::{Context, Driver, DriverKey};

use crate::{
    Connection, Match, Search, State, WithPath,
    composite::{Composite, MatchedComposite, SearchableComposite},
    instance::Instance,
    netlist::SearchableNetlist,
    queries::netlist::basic::{and::AndGate, dff::Sdffe},
};
use itertools::iproduct;

#[derive(Debug, Clone)]
pub struct SdffeThenAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub sdffe: Sdffe<S>,
    pub andg: AndGate<S>,
}

impl<S> SdffeThenAnd<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            sdffe: Sdffe::new(path.child("sdffe".to_string())),
            andg: AndGate::new(path.child("andg".to_string())),
        }
    }
}

impl<S> WithPath<S> for SdffeThenAnd<S>
where
    S: State,
{
    crate::impl_find_port!(SdffeThenAnd, sdffe, andg);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for SdffeThenAnd<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        // Define the connection: sdffe.q -> (andg.a OR andg.b)
        vec![vec![
            Connection {
                from: self.sdffe.q.clone(),
                to: self.andg.a.clone(),
            },
            Connection {
                from: self.sdffe.q.clone(),
                to: self.andg.b.clone(),
            },
        ]]
    }
}

impl<'ctx> MatchedComposite<'ctx> for SdffeThenAnd<Match<'ctx>> {}

impl SearchableComposite for SdffeThenAnd<Search> {
    type Hit<'ctx> = SdffeThenAnd<Match<'ctx>>;

    fn context(driver: &Driver) -> Result<Context, Box<dyn std::error::Error>> {
        let sdffe_context = Sdffe::<Search>::context(driver)?;
        let and_context = AndGate::<Search>::context(driver)?;
        Ok(sdffe_context.merge(and_context))
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        // First get individual matches
        let sdffe_matches = Sdffe::<Search>::query(
            haystack_key,
            context,
            path.child("sdffe".to_string()),
            config,
        );
        let and_matches = AndGate::<Search>::query(
            haystack_key,
            context,
            path.child("andg".to_string()),
            config,
        );

        // Create composite instances

        iproduct!(sdffe_matches, and_matches)
            .map(|(sdffe, andg)| SdffeThenAnd {
                path: path.clone(),
                sdffe,
                andg,
            })
            .filter(|composite| composite.validate_connections(composite.connections()))
            .collect::<Vec<_>>()
    }
}
