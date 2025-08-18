use svql_driver::SubgraphMatch;
use svql_driver::subgraph::cell_kind::CellWrapper;

use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{Match, Search, State, Wire, WithPath};

// Direction is implicit from which helper we call (input vs output),
// so we don’t need a direction enum here. For multi-bit ports, iterate bits.

#[derive(Debug, Clone)]
pub struct And<S>
where
    S: State,
{
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S> WithPath<S> for And<S>
where
    S: State,
{
    crate::impl_find_port!(And, a, b, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Netlist<S> for And<S>
where
    S: State,
{
    const MODULE_NAME: &'static str = "and_gate";
    const FILE_PATH: &'static str = "./examples/patterns/basic/and/and.v";
}

impl SearchableNetlist for And<Search> {
    type Hit<'p, 'd> = And<Match<'p, 'd>>;

    fn from_query_match<'p, 'd>(m: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit<'p, 'd> {
        // Single-bit ports; for multi-bit, iterate 0..width
        let a_match = bind_input(m, "a", 0);
        let b_match = bind_input(m, "b", 0);
        let y_match = bind_output(m, "y", 0);

        And {
            path: path.clone(),
            a: Wire::with_val(path.child("a".to_string()), a_match),
            b: Wire::with_val(path.child("b".to_string()), b_match),
            y: Wire::with_val(path.child("y".to_string()), y_match),
        }
    }
}

// Helpers: turn SubgraphMatch into our Match<'p,'d> for a single bit.
// For wider ports, call with bit = 0..width and collect into Vec<Wire<_>>.

fn bind_input<'p, 'd>(m: &SubgraphMatch<'p, 'd>, name: &str, bit: usize) -> Match<'p, 'd> {
    let pat = m.input_by_name.get(name).copied();
    let des = m.design_source_of_input_bit(name, bit).map(|(c, _b)| c);
    Match {
        pat_cell_ref: pat,
        design_cell_ref: des,
    }
}

fn bind_output<'p, 'd>(m: &SubgraphMatch<'p, 'd>, name: &str, bit: usize) -> Match<'p, 'd> {
    let pat = m.output_by_name.get(name).copied();
    let des = m.design_driver_of_output_bit(name, bit).map(|(c, _b)| c);
    Match {
        pat_cell_ref: pat,
        design_cell_ref: des,
    }
}
