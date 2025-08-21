use svql_driver::{DriverKey, context::Context, driver::Driver};
use svql_subgraph::Config;

use crate::{
    Match, Search, State, Wire, WithPath,
    instance::Instance,
    queries::netlist::basic::and::{and_gate::AndGate, and_mux::AndMux, and_nor::AndNor},
};

#[derive(Debug, Clone)]
pub enum AndAny<S>
where
    S: State,
{
    Gate(AndGate<S>),
    Mux(AndMux<S>),
    Nor(AndNor<S>),
}

impl<S> WithPath<S> for AndAny<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        match self {
            AndAny::Gate(inner) => inner.find_port(p),
            AndAny::Mux(inner) => inner.find_port(p),
            AndAny::Nor(inner) => inner.find_port(p),
        }
    }

    fn path(&self) -> Instance {
        match self {
            AndAny::Gate(inner) => inner.path(),
            AndAny::Mux(inner) => inner.path(),
            AndAny::Nor(inner) => inner.path(),
        }
    }
}

impl AndAny<Search> {
    pub fn context(driver: &Driver) -> Context {
        // Merge contexts from all child netlists
        let gate_ctx = AndGate::<Search>::context(driver, "and_gate");
        let mux_ctx = AndMux::<Search>::context(driver, "and_mux");
        let nor_ctx = AndNor::<Search>::context(driver, "and_nor");

        gate_ctx.merge(mux_ctx).merge(nor_ctx)
    }

    /// Unified query across and_gate, and_mux, and and_nor.
    /// Uses gates-only dedupe to collapse matches that differ only by IO bindings.
    pub fn query<'p, 'd>(
        pattern_keys: &[DriverKey], // [and_gate_key, and_mux_key, and_nor_key]
        haystack_key: &DriverKey,
        context: &Context,
        path: Instance,
        config: &Config,
    ) -> Vec<AndAny<Match<'p, 'd>>> {
        let mut out: Vec<AndAny<Match<'p, 'd>>> = Vec::new();

        // Query each variant
        if let Some(gate_key) = pattern_keys.get(0) {
            let gate_hits =
                AndGate::<Search>::query(gate_key, haystack_key, context, path.clone(), config);
            out.extend(gate_hits.into_iter().map(AndAny::Gate));
        }

        if let Some(mux_key) = pattern_keys.get(1) {
            let mux_hits =
                AndMux::<Search>::query(mux_key, haystack_key, context, path.clone(), config);
            out.extend(mux_hits.into_iter().map(AndAny::Mux));
        }

        if let Some(nor_key) = pattern_keys.get(2) {
            let nor_hits =
                AndNor::<Search>::query(nor_key, haystack_key, context, path.clone(), config);
            out.extend(nor_hits.into_iter().map(AndAny::Nor));
        }

        out
    }
}
