use svql_driver::prelude::Driver;
use svql_driver::query_ctx::QueryCtx;

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

impl SearchableNetlist for AndMux<Search> {
    type Hit<'p, 'd> = AndMux<Match<'p, 'd>>;

    fn from_subgraph<'p, 'd>(
        m: &svql_subgraph::SubgraphMatch<'p, 'd>,
        path: Instance,
    ) -> Self::Hit<'p, 'd> {
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

impl AndMux<Search> {
    pub fn query<'ctx>(
        ctx: &'ctx QueryCtx,
        path: Instance,
        config: &svql_subgraph::config::Config,
    ) -> Vec<AndMux<Match<'ctx, 'ctx>>> {
        <Self as SearchableNetlist>::query(ctx, path, config)
    }
}
