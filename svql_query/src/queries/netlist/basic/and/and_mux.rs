// svql_query/src/queries/netlist/basic/and/and_mux.rs

use svql_subgraph::SubgraphMatch;

use crate::binding::{bind_input, bind_output};
use crate::instance::Instance;
use crate::netlist::{NetlistMeta, PortDir, PortSpec, SearchableNetlist};
use crate::{Match, Search, State, Wire, WithPath};

#[derive(Debug, Clone)]
pub struct AndMux<S>
where
    S: State,
{
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S> AndMux<S>
where
    S: State,
{
    /// Uniform constructor so composites can assemble a Search-only shape.
    pub fn new(path: Instance) -> Self {
        AndMux {
            path: path.clone(),
            a: crate::Wire::new(path.child("a".to_string())),
            b: crate::Wire::new(path.child("b".to_string())),
            y: crate::Wire::new(path.child("y".to_string())),
        }
    }
}

impl<S> WithPath<S> for AndMux<S>
where
    S: State,
{
    crate::impl_find_port!(AndMux, a, b, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl NetlistMeta for AndMux<Search> {
    const MODULE_NAME: &'static str = "and_mux";
    const FILE_PATH: &'static str = "examples/patterns/basic/and/verilog/and_mux.v";

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
impl SearchableNetlist for AndMux<Search> {
    type Hit<'ctx> = AndMux<Match<'ctx>>;

    fn from_subgraph<'ctx>(m: &SubgraphMatch<'ctx, 'ctx>, path: Instance) -> Self::Hit<'ctx> {
        let a_match = bind_input(m, "a", 0);
        let b_match = bind_input(m, "b", 0);
        let y_match = bind_output(m, "y", 0);

        AndMux {
            path: path.clone(),
            a: Wire::with_val(path.child("a".to_string()), a_match),
            b: Wire::with_val(path.child("b".to_string()), b_match),
            y: Wire::with_val(path.child("y".to_string()), y_match),
        }
    }
}
