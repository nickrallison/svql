use std::path::Path;

use itertools::iproduct;
use svql_driver::driver::Driver;
use svql_subgraph::Config;

use crate::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::instance::Instance;
use crate::{Match, Search, State, Wire, WithPath};

use crate::queries::netlist::basic::and::and_gate::AndGate;
use crate::queries::netlist::basic::dff::Sdffe;

/// DFF -> AND composite:
/// Require that sdffe.q drives either and_gate.a or and_gate.b (one must hold).
#[derive(Debug, Clone)]
pub struct DffThenAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub sdffe: Sdffe<S>,
    pub and_gate: AndGate<S>,
}

impl<S> DffThenAnd<S>
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

impl<S> WithPath<S> for DffThenAnd<S>
where
    S: State,
{
    crate::impl_find_port!(DffThenAnd, sdffe, and_gate);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for DffThenAnd<S>
where
    S: State,
{
    /// Single OR-set of connections: sdffe.q -> and_gate.a OR sdffe.q -> and_gate.b
    fn connections(&self) -> Vec<Vec<crate::Connection<S>>> {
        vec![vec![
            crate::Connection {
                from: self.sdffe.q.clone(),
                to: self.and_gate.a.clone(),
            },
            crate::Connection {
                from: self.sdffe.q.clone(),
                to: self.and_gate.b.clone(),
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
        driver: &'ctx Driver,
        haystack_module_name: &str,
        haystack_path: &Path,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx, 'ctx>> {
        // Borrow child contexts from the pool (must have been ensured by the caller).
        let and_results = AndGate::query(
            driver,
            haystack_module_name,
            haystack_path,
            path.child("and_gate".to_string()),
            config,
        );
        let sdffe_results = Sdffe::query(
            driver,
            haystack_module_name,
            haystack_path,
            path.child("sdffe".to_string()),
            config,
        );

        let results = iproduct!(and_results, sdffe_results)
            .map(|(and_gate, sdffe)| Self::Hit {
                and_gate,
                sdffe,
                path: path.clone(),
            })
            .filter(|s| {
                let conn_ok = s.validate_connections(s.connections());
                let other_ok = s.other_filters().iter().all(|f| f(s));
                conn_ok && other_ok
            })
            .collect::<Vec<_>>();

        results
    }
}
