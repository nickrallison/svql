// svql_query/src/queries/enum_composite/and_any.rs

use std::result;

use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::Config;

use crate::{
    Match, Search, State, Wire, WithPath,
    composite::{EnumComposite, MatchedEnumComposite, SearchableEnumComposite},
    instance::Instance,
    netlist::SearchableNetlist,
    queries::netlist::basic::and::{AndGate, AndMux, AndNor},
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

// As a simple container, no connections are enforced.
// This mirrors the EnumComposite trait surface but remains a no-op.
impl<S> EnumComposite<S> for AndAny<S> where S: State {}

impl<'ctx> MatchedEnumComposite<'ctx> for AndAny<Match<'ctx>> {}

impl SearchableEnumComposite for AndAny<Search> {
    type Hit<'ctx> = AndAny<Match<'ctx>>;

    fn context(driver: &Driver) -> Result<Context, Box<dyn std::error::Error>> {
        let and_gate_context = AndGate::<Search>::context(driver)?;
        let and_mux_context = AndMux::<Search>::context(driver)?;
        let and_nor_context = AndNor::<Search>::context(driver)?;

        let context = and_gate_context
            .merge(and_mux_context)
            .merge(and_nor_context);

        Ok(context)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        let and_gate_matches = AndGate::<Search>::query(
            haystack_key,
            context,
            path.child("and_gate".to_string()),
            config,
        )
        .iter()
        .map(|match_| AndAny::<Match<'ctx>>::Gate(match_.clone()))
        .collect::<Vec<_>>();

        let and_mux_matches = AndMux::<Search>::query(
            haystack_key,
            context,
            path.child("and_mux".to_string()),
            config,
        )
        .iter()
        .map(|match_| AndAny::<Match<'ctx>>::Mux(match_.clone()))
        .collect::<Vec<_>>();

        let and_nor_matches = AndNor::<Search>::query(
            haystack_key,
            context,
            path.child("and_nor".to_string()),
            config,
        )
        .iter()
        .map(|match_| AndAny::<Match<'ctx>>::Nor(match_.clone()))
        .collect::<Vec<_>>();

        // Create composite instances

        let results = and_gate_matches
            .into_iter()
            .chain(and_mux_matches)
            .chain(and_nor_matches)
            .collect::<Vec<_>>();
        results
    }
}

#[cfg(test)]
mod tests {
    use log::trace;
    use svql_driver::Driver;
    use svql_subgraph::{Config, DedupeMode};

    use crate::{
        Search, composite::SearchableEnumComposite, instance::Instance,
        queries::enum_composite::and_any::AndAny,
    };

    #[test]
    fn test_and_any() {
        let driver = Driver::new_workspace().expect("Failed to create driver");

        let context =
            AndAny::<Search>::context(&driver).expect("Failed to create context for AndAny");

        // haystack
        let haystack_path = "examples/fixtures/basic/and/json/mixed_and_tree.json";
        let haystack_module_name = "mixed_and_tree";
        let (haystack_key, haystack) = driver
            .get_or_load_design(haystack_path, haystack_module_name.to_string())
            .expect("Failed to get haystack design");

        let context = context.with_design(haystack_key.clone(), haystack.clone());

        let config = Config::builder()
            .exact_length()
            .dedupe(DedupeMode::AutoMorph)
            .build();

        // root path for the composite
        let root = Instance::root("and_any".to_string());

        // run composite query
        let hits = AndAny::<Search>::query(&haystack_key, &context, root, &config);

        for h in &hits {
            trace!("Found match: {:#?}", h);
        }

        let mut gate_cnt = 0usize;
        let mut mux_cnt = 0usize;
        let mut nor_cnt = 0usize;

        for h in hits {
            match h {
                AndAny::Gate(_) => gate_cnt += 1,
                AndAny::Mux(_) => mux_cnt += 1,
                AndAny::Nor(_) => nor_cnt += 1,
            }
        }

        trace!(
            "Found {} gate matches, {} mux matches, {} nor matches",
            gate_cnt, mux_cnt, nor_cnt
        );

        assert_eq!(gate_cnt, 3, "expected 3 and_gate matches");
        assert_eq!(mux_cnt, 2, "expected 2 and_mux matches");
        assert_eq!(nor_cnt, 2, "expected 2 and_nor matches");
    }
}
