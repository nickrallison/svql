use itertools::iproduct;
use svql_driver::cache::Cache;
use svql_driver::prelude::Driver;

use svql_driver::util::load_driver_cached;
use svql_query::composite::{Composite, MatchedComposite, SearchableComposite};
use svql_query::instance::Instance;
use svql_query::{Connection, Match, Search, State, WithPath};
use svql_query::{Wire, impl_find_port};

use svql_query::queries::netlist::and::And;
use svql_query::queries::netlist::dff::Sdffe;

/// DFF -> AND composite:
/// - Require that sdffe.q drives either and.a or and.b (one must hold).
#[derive(Debug, Clone)]
pub struct DffThenAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub sdffe: Sdffe<S>,
    pub andg: And<S>,
}

impl<S> WithPath<S> for DffThenAnd<S>
where
    S: State,
{
    impl_find_port!(DffThenAnd, sdffe, andg);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> DffThenAnd<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            sdffe: Sdffe::new(path.child("sdffe".to_string())),
            andg: And::new(path.child("andg".to_string())),
        }
    }
}

impl<S> Composite<S> for DffThenAnd<S>
where
    S: State,
{
    /// Single OR-set of connections: sdffe.q -> andg.a OR sdffe.q -> andg.b
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut sets: Vec<Vec<Connection<S>>> = Vec::new();
        let mut one_or_set: Vec<Connection<S>> = Vec::new();

        one_or_set.push(Connection {
            from: self.sdffe.q.clone(),
            to: self.andg.a.clone(),
        });
        one_or_set.push(Connection {
            from: self.sdffe.q.clone(),
            to: self.andg.b.clone(),
        });

        sets.push(one_or_set);
        sets
    }
}

impl SearchableComposite for DffThenAnd<Search> {
    type Hit<'p, 'd> = DffThenAnd<Match<'p, 'd>>;
}

impl<'p, 'd> MatchedComposite<'p, 'd> for DffThenAnd<Match<'p, 'd>> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
        // No extra user filters for the demo
        vec![]
    }
}

impl DffThenAnd<Search> {
    /// Composite-specific, macro-friendly query signature:
    /// pass the two pattern drivers (sdffe, and) and the haystack.
    pub fn query<'p, 'd>(
        sdffe_pattern: &'p Driver,
        and_pattern: &'p Driver,
        haystack: &'d Driver,
        path: Instance,
    ) -> Vec<DffThenAnd<Match<'p, 'd>>> {
        // Sub-queries at stable locations in the instance tree
        let sdffe_hits: Vec<Sdffe<Match<'p, 'd>>> =
            svql_query::queries::netlist::dff::Sdffe::<Search>::query(
                sdffe_pattern,
                haystack,
                path.child("sdffe".to_string()),
            );
        let and_hits: Vec<And<Match<'p, 'd>>> =
            svql_query::queries::netlist::and::And::<Search>::query(
                and_pattern,
                haystack,
                path.child("andg".to_string()),
            );

        // Cartesian product + map to composite + filter by connectivity and user filters
        iproduct!(sdffe_hits, and_hits)
            .map(|(sdffe, andg)| DffThenAnd::<Match> {
                path: path.clone(),
                sdffe,
                andg,
            })
            .filter(|hit| {
                let conn_ok = hit.validate_connections(hit.connections());
                let other_ok = hit.other_filters().iter().all(|f| f(hit));
                conn_ok && other_ok
            })
            .collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let mut cache = Cache::new();

    let and_gate_driver = load_driver_cached("examples/patterns/basic/and/and_gate.v", &mut cache)?;
    let sdffe_driver = load_driver_cached("examples/patterns/basic/ff/sdffe.v", &mut cache)?;
    // haystack
    let haystack = load_driver_cached(
        "examples/patterns/basic/ff/and_q_double_sdffe.v",
        &mut cache,
    )?;

    // root path for the composite
    let root = Instance::root("dff_then_and".to_string());

    // run composite query
    let hits = DffThenAnd::<Search>::query(&sdffe_driver, &and_gate_driver, &haystack, root);

    // There are two DFFs whose Q each feed the AND inputs; we expect 2 matches.
    assert_eq!(hits.len(), 2, "expected 2 DffThenAnd matches");

    // Sanity: validate bindings exist on each hit
    for h in &hits {
        let q = h
            .sdffe
            .q
            .val
            .as_ref()
            .expect("missing sdffe.q binding")
            .design_cell_ref
            .expect("sdffe.q should have a design driver");

        // One of these must be driven by the same q source cell
        let a_src = h
            .andg
            .a
            .val
            .as_ref()
            .expect("missing andg.a binding")
            .design_cell_ref
            .expect("andg.a should have a design source");
        let b_src = h
            .andg
            .b
            .val
            .as_ref()
            .expect("missing andg.b binding")
            .design_cell_ref
            .expect("andg.b should have a design source");

        assert!(
            q == a_src || q == b_src,
            "expected sdffe.q to drive either andg.a or andg.b"
        );
    }

    Ok(())
}
