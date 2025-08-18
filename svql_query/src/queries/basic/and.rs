use svql_driver::prelude::Driver;

use crate::binding::{bind_input, bind_output};
use crate::instance::Instance;
use crate::netlist::{NetlistMeta, PortDir, PortSpec, SearchableNetlist};
use crate::{Match, Search, State, Wire, WithPath};

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

// Static metadata for codegen/introspection.
// A define_netlist! macro would generate this from the declarative input.
impl NetlistMeta for And<Search> {
    const MODULE_NAME: &'static str = "and_gate";
    const FILE_PATH: &'static str = "examples/patterns/basic/and/and_gate.v";

    const PORTS: &'static [PortSpec] = &[
        PortSpec {
            name: "a",
            dir: PortDir::In,
        },
        PortSpec {
            name: "b",
            dir: PortDir::In,
        },
        PortSpec {
            name: "y",
            dir: PortDir::Out,
        },
    ];
}

// The query surface. A macro can generate this impl (and the inherent
// wrapper below) for any netlist with the same shape.
impl SearchableNetlist for And<Search> {
    type Hit<'p, 'd> = And<Match<'p, 'd>>;

    fn from_subgraph<'p, 'd>(
        m: &svql_subgraph::SubgraphMatch<'p, 'd>,
        path: Instance,
    ) -> Self::Hit<'p, 'd> {
        // Single‑bit ports in this example; multi‑bit support would iterate 0..width.
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

// Inherent shim so callers can write And::<Search>::query(...) without UFCS.
// A macro can also emit this block verbatim for each netlist type.
impl And<Search> {
    pub fn query<'p, 'd>(
        pattern: &'p Driver,
        haystack: &'d Driver,
        path: Instance,
    ) -> Vec<And<Match<'p, 'd>>> {
        <Self as SearchableNetlist>::query(pattern, haystack, path)
    }
}
