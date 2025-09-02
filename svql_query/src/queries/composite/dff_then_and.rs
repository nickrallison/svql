// svql_query/src/queries/composite/dff_then_and.rs
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

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
    pub and_gate: AndGate<S>,
}

impl<S> SdffeThenAnd<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            sdffe: Sdffe::new(path.child("sdffe".to_string())),
            and_gate: AndGate::new(path.child("and_gate".to_string())),
        }
    }
}

impl<S> WithPath<S> for SdffeThenAnd<S>
where
    S: State,
{
    crate::impl_find_port!(SdffeThenAnd, sdffe, and_gate);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for SdffeThenAnd<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        // Define the connection: sdffe.q -> (and_gate.a OR and_gate.b)
        vec![vec![
            Connection {
                from: self.sdffe.q.clone(),
                to: self.and_gate.a.clone(),
            },
            Connection {
                from: self.sdffe.q.clone(),
                to: self.and_gate.b.clone(),
            },
        ]]
    }
}

impl<'ctx> MatchedComposite<'ctx> for SdffeThenAnd<Match<'ctx>> {}

impl SearchableComposite for SdffeThenAnd<Search> {
    type Hit<'ctx> = SdffeThenAnd<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        let sdffe_context = Sdffe::<Search>::context(driver, config)?;
        let and_context = AndGate::<Search>::context(driver, config)?;
        Ok(sdffe_context.merge(and_context))
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        // Run individual queries in parallel when Rayon is enabled
        #[cfg(feature = "rayon")]
        {
            tracing::event!(
                tracing::Level::INFO,
                "SdffeThenAnd::query: executing with Rayon parallel queries"
            );

            let (sdffe_matches, and_matches) = rayon::join(
                || {
                    Sdffe::<Search>::query(
                        haystack_key,
                        context,
                        path.child("sdffe".to_string()),
                        config,
                    )
                },
                || {
                    AndGate::<Search>::query(
                        haystack_key,
                        context,
                        path.child("and_gate".to_string()),
                        config,
                    )
                },
            );

            // Create composite instances
            iproduct!(sdffe_matches, and_matches)
                .par_bridge()
                .map(|(sdffe, and_gate)| SdffeThenAnd {
                    path: path.clone(),
                    sdffe,
                    and_gate,
                })
                .filter(|composite| composite.validate_connections(composite.connections()))
                .collect::<Vec<_>>()
        }

        #[cfg(not(feature = "rayon"))]
        {
            tracing::event!(
                tracing::Level::INFO,
                "SdffeThenAnd::query: executing sequential queries"
            );

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
                path.child("and_gate".to_string()),
                config,
            );

            // Create composite instances
            iproduct!(sdffe_matches, and_matches)
                .map(|(sdffe, and_gate)| SdffeThenAnd {
                    path: path.clone(),
                    sdffe,
                    and_gate,
                })
                .filter(|composite| composite.validate_connections(composite.connections()))
                .collect::<Vec<_>>()
        }
    }
}
