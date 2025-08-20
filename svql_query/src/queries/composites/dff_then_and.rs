use crate::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::haystack::HaystackPool;
use crate::instance::Instance;
use crate::{Match, Search, State, Wire, WithPath};

use crate::queries::netlist::basic::and::and_gate::AndGate;
use crate::queries::netlist::basic::dff::Sdffe;

/// DFF -> AND composite:
/// Require that sdffe.q drives either andg.a or andg.b (one must hold).
#[derive(Debug, Clone)]
pub struct DffThenAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub sdffe: Sdffe<S>,
    pub andg: AndGate<S>,
}

impl<S> DffThenAnd<S>
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

impl<S> WithPath<S> for DffThenAnd<S>
where
    S: State,
{
    crate::impl_find_port!(DffThenAnd, sdffe, andg);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for DffThenAnd<S>
where
    S: State,
{
    /// Single OR-set of connections: sdffe.q -> andg.a OR sdffe.q -> andg.b
    fn connections(&self) -> Vec<Vec<crate::Connection<S>>> {
        vec![vec![
            crate::Connection {
                from: self.sdffe.q.clone(),
                to: self.andg.a.clone(),
            },
            crate::Connection {
                from: self.sdffe.q.clone(),
                to: self.andg.b.clone(),
            },
        ]]
    }
}

impl<'p, 'd> MatchedComposite<'p, 'd> for DffThenAnd<Match<'p, 'd>> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
        Vec::new()
    }

    /// Override: compare only design endpoints; patterns differ across sub-queries.
    fn validate_connection(&self, connection: crate::Connection<Match<'p, 'd>>) -> bool {
        let from_wire = self.find_port(&connection.from.path);
        let to_wire = self.find_port(&connection.to.path);

        match (from_wire, to_wire) {
            (Some(from), Some(to)) => {
                let fm = from.val.as_ref();
                let tm = to.val.as_ref();

                match (fm, tm) {
                    (Some(fm), Some(tm)) => {
                        let from_cell = fm.design_cell_ref;
                        let to_cell = tm.design_cell_ref;
                        from_cell.is_some() && to_cell.is_some() && from_cell == to_cell
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl SearchableComposite for DffThenAnd<Search> {
    type Hit<'p, 'd> = DffThenAnd<Match<'p, 'd>>;

    fn query<'ctx>(
        hay: &'ctx HaystackPool,
        path: Instance,
        config: &svql_subgraph::config::Config,
    ) -> Vec<Self::Hit<'ctx, 'ctx>> {
        // Borrow child contexts from the pool (must have been ensured by the caller).
        let sdffe_ctx = hay.get::<Sdffe<Search>>();
        let and_ctx = hay.get::<AndGate<Search>>();

        // Run sub-queries under their respective contexts.
        let sdffe_hits = Sdffe::<Search>::query(sdffe_ctx, path.child("sdffe".to_string()), config);
        let and_hits = AndGate::<Search>::query(and_ctx, path.child("andg".to_string()), config);

        // Assemble candidates and validate connections.
        let mut out: Vec<Self::Hit<'ctx, 'ctx>> = Vec::new();

        for s in &sdffe_hits {
            for a in &and_hits {
                let cand = DffThenAnd::<Match> {
                    path: path.clone(),
                    sdffe: s.clone(),
                    andg: a.clone(),
                };
                if cand.validate_connections(cand.connections())
                    && cand.other_filters().iter().all(|f| f(&cand))
                {
                    out.push(cand);
                }
            }
        }

        out
    }
}
