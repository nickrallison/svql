use log::trace;
use svql_driver::cache::Cache;
use svql_driver::prelude::Driver;

use svql_driver::util::load_driver_cached;
use svql_query::composite::{Composite, MatchedComposite, SearchableComposite};
use svql_query::impl_find_port;
use svql_query::instance::Instance;
use svql_query::{Connection, Match, Search, State, WithPath};

use svql_query::queries::netlist::basic::and::And;
use svql_query::queries::netlist::basic::dff::Sdffe;

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

    /// OVERRIDE: Compare only the design endpoints (drivers/sources).
    /// The default equality on Match would compare both pattern and design cells,
    /// which cannot succeed across different sub-netlists inside a composite.
    fn validate_connection(&self, connection: Connection<Match<'p, 'd>>) -> bool {
        let from_wire = self.find_port(&connection.from.path);
        let to_wire = self.find_port(&connection.to.path);

        match (from_wire, to_wire) {
            (Some(from), Some(to)) => {
                let from_m = from.val.as_ref();
                let to_m = to.val.as_ref();

                match (from_m, to_m) {
                    (Some(fm), Some(tm)) => {
                        let from_cell = fm.design_cell_ref;
                        let to_cell = tm.design_cell_ref;

                        trace!(
                            "validate_connection: from={} to={} => from_cell={:?} to_cell={:?}",
                            connection.from.path.inst_path(),
                            connection.to.path.inst_path(),
                            from_cell.as_ref().map(|c| c.debug_index()),
                            to_cell.as_ref().map(|c| c.debug_index())
                        );

                        let ok = from_cell.is_some() && to_cell.is_some() && from_cell == to_cell;
                        if !ok {
                            trace!(
                                "validate_connection: REJECT: design endpoints do not match (or missing)"
                            );
                        } else {
                            trace!("validate_connection: ACCEPT");
                        }
                        ok
                    }
                    _ => {
                        trace!(
                            "validate_connection: REJECT: missing Match values: from_val_present={} to_val_present={}",
                            from_m.is_some(),
                            to_m.is_some()
                        );
                        false
                    }
                }
            }
            (f, t) => {
                trace!(
                    "validate_connection: REJECT: could not resolve ports: from_found={} to_found={}",
                    f.is_some(),
                    t.is_some()
                );
                false
            }
        }
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
            svql_query::queries::netlist::basic::dff::Sdffe::<Search>::query(
                sdffe_pattern,
                haystack,
                path.child("sdffe".to_string()),
            );
        let and_hits: Vec<And<Match<'p, 'd>>> =
            svql_query::queries::netlist::basic::and::And::<Search>::query(
                and_pattern,
                haystack,
                path.child("andg".to_string()),
            );

        trace!(
            "DffThenAnd.query: sdffe_hits={}, and_hits={}",
            sdffe_hits.len(),
            and_hits.len()
        );

        // Instead of a purely functional pipeline, use loops so we can add detailed traces.
        let mut out: Vec<DffThenAnd<Match<'p, 'd>>> = Vec::new();

        for (i, s) in sdffe_hits.into_iter().enumerate() {
            // Extract the sdffe.q design endpoint (if any) for logging
            let s_q = s.q.val.as_ref().and_then(|m| m.design_cell_ref);
            trace!(
                "  sdffe[{}]: q.design_cell_ref={:?}",
                i,
                s_q.as_ref().map(|c| c.debug_index())
            );

            for (j, a) in and_hits.clone().into_iter().enumerate() {
                let a_a = a.a.val.as_ref().and_then(|m| m.design_cell_ref);
                let a_b = a.b.val.as_ref().and_then(|m| m.design_cell_ref);

                trace!(
                    "    and[{}]: a.design_cell_ref={:?} b.design_cell_ref={:?}",
                    j,
                    a_a.as_ref().map(|c| c.debug_index()),
                    a_b.as_ref().map(|c| c.debug_index())
                );

                let cand = DffThenAnd::<Match> {
                    path: path.clone(),
                    sdffe: s.clone(),
                    andg: a.clone(),
                };

                let conn_ok = cand.validate_connections(cand.connections());
                let other_ok = cand.other_filters().iter().all(|f| f(&cand));

                trace!(
                    "    candidate(sdffe={}, and={}): conn_ok={} other_ok={}",
                    i, j, conn_ok, other_ok
                );

                if conn_ok && other_ok {
                    trace!("    => ACCEPT candidate");
                    out.push(cand);
                } else {
                    trace!("    => REJECT candidate");
                }
            }
        }

        trace!("DffThenAnd.query: total accepted matches={}", out.len());
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
    let sdffe_driver =
        load_driver_cached("examples/patterns/basic/ff/verilog/sdffe.v", &mut cache)?;
    // haystack
    let haystack = load_driver_cached(
        "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
        &mut cache,
    )?;

    // root path for the composite
    let root = Instance::root("dff_then_and".to_string());

    // run composite query
    let hits = DffThenAnd::<Search>::query(&sdffe_driver, &and_gate_driver, &haystack, root);

    trace!("main: DffThenAnd matches={}", hits.len());

    // There are two DFFs whose Q each feed the AND inputs; we expect 2 matches.
    assert_eq!(hits.len(), 2, "expected 2 DffThenAnd matches");

    // Sanity: validate bindings exist on each hit
    for (k, h) in hits.iter().enumerate() {
        let q = h
            .sdffe
            .q
            .val
            .as_ref()
            .expect("missing sdffe.q binding")
            .design_cell_ref
            .expect("sdffe.q should have a design driver");

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

        trace!(
            "hit[{}]: q={:?} a_src={:?} b_src={:?}",
            k,
            q.debug_index(),
            a_src.debug_index(),
            b_src.debug_index()
        );

        assert!(
            q == a_src || q == b_src,
            "expected sdffe.q to drive either andg.a or andg.b"
        );

        // println entire match

        println!("hit[{}]: {:#?}", k, h);
    }

    Ok(())
}
