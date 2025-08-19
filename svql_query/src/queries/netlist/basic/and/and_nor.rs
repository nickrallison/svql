use svql_driver::prelude::Driver;

use crate::binding::{bind_input, bind_output};
use crate::instance::Instance;
use crate::netlist::{NetlistMeta, PortDir, PortSpec, SearchableNetlist};
use crate::{Match, Search, State, Wire, WithPath};

#[derive(Debug, Clone)]
pub struct AndNor<S>
where
    S: State,
{
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S> AndNor<S>
where
    S: State,
{
    /// Uniform constructor so composites can assemble a Search-only shape.
    pub fn new(path: Instance) -> Self {
        AndNor {
            path: path.clone(),
            a: crate::Wire::new(path.child("a".to_string())),
            b: crate::Wire::new(path.child("b".to_string())),
            y: crate::Wire::new(path.child("y".to_string())),
        }
    }
}

impl<S> WithPath<S> for AndNor<S>
where
    S: State,
{
    crate::impl_find_port!(AndNor, a, b, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

// Static metadata for codegen/introspection.
impl NetlistMeta for AndNor<Search> {
    const MODULE_NAME: &'static str = "and_nor";
    // Keep consistency with And<Search>::FILE_PATH (no `/verilog/` in the path).
    const FILE_PATH: &'static str = "examples/patterns/basic/and/and_nor.v";

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

// Query surface; mirrors the implementation used for And<Search>.
impl SearchableNetlist for AndNor<Search> {
    type Hit<'p, 'd> = AndNor<Match<'p, 'd>>;

    fn from_subgraph<'p, 'd>(
        m: &svql_subgraph::SubgraphMatch<'p, 'd>,
        path: Instance,
    ) -> Self::Hit<'p, 'd> {
        let a_match = bind_input(m, "a", 0);
        let b_match = bind_input(m, "b", 0);
        let y_match = bind_output(m, "y", 0);

        AndNor {
            path: path.clone(),
            a: Wire::with_val(path.child("a".to_string()), a_match),
            b: Wire::with_val(path.child("b".to_string()), b_match),
            y: Wire::with_val(path.child("y".to_string()), y_match),
        }
    }
}

// Inherent shim for ergonomic call sites.
impl AndNor<Search> {
    pub fn query<'p, 'd>(
        pattern: &'p Driver,
        haystack: &'d Driver,
        path: Instance,
        config: &svql_subgraph::config::Config,
    ) -> Vec<AndNor<Match<'p, 'd>>> {
        <Self as SearchableNetlist>::query(pattern, haystack, path, config)
    }
}
