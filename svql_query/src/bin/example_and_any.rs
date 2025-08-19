use log::trace;
use svql_driver::cache::Cache;
use svql_driver::prelude::Driver;
use svql_driver::util::load_driver_cached;

use svql_query::instance::Instance;
use svql_query::queries::netlist::basic::and::{
    and_gate::AndGate, and_mux::AndMux, and_nor::AndNor,
};
use svql_query::{Match, Search, State, Wire, WithPath};

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
    /// Run a unified query that accepts any of:
    /// - and_gate
    /// - and_mux
    /// - and_nor
    ///
    /// Each inner query is invoked with the same root path so the wrapped
    /// variants are interchangeable in composites or further processing.
    pub fn query<'p, 'd>(
        and_gate_pattern: &'p Driver,
        and_mux_pattern: &'p Driver,
        and_nor_pattern: &'p Driver,
        haystack: &'d Driver,
        path: Instance,
    ) -> Vec<AndAny<Match<'p, 'd>>> {
        let mut out: Vec<AndAny<Match<'p, 'd>>> = Vec::new();

        let gate_hits = AndGate::<Search>::query(and_gate_pattern, haystack, path.clone());
        out.extend(gate_hits.into_iter().map(AndAny::Gate));

        let mux_hits = AndMux::<Search>::query(and_mux_pattern, haystack, path.clone());
        out.extend(mux_hits.into_iter().map(AndAny::Mux));

        let nor_hits = AndNor::<Search>::query(and_nor_pattern, haystack, path.clone());
        out.extend(nor_hits.into_iter().map(AndAny::Nor));

        out
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut cache = Cache::new();

    let and_gate_driver =
        load_driver_cached("examples/patterns/basic/and/verilog/and_gate.v", &mut cache)?;
    let and_mux_driver =
        load_driver_cached("examples/patterns/basic/and/verilog/and_mux.v", &mut cache)?;
    let and_nor_driver =
        load_driver_cached("examples/patterns/basic/and/verilog/and_nor.v", &mut cache)?;
    // haystack
    let haystack = load_driver_cached(
        "examples/fixtures/basic/and/json/mixed_and_tree.json",
        &mut cache,
    )?;

    // root path for the composite
    let root = Instance::root("dff_then_and".to_string());

    // run composite query
    let hits = AndAny::<Search>::query(
        &and_gate_driver,
        &and_mux_driver,
        &and_nor_driver,
        &haystack,
        root,
    );

    // trace!("main: AndAny matches={}", hits.len());

    for h in &hits {
        trace!("Found match: {:#?}", h);
    }

    // assert_eq!(
    //     hits.len(),
    //     7,
    //     "expected 7 total and matches in mixed_and_tree"
    // );

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

    Ok(())
}
