// svql_query/src/queries/netlist/basic/dff.rs
use svql_driver::driver::Driver;

use crate::binding::{bind_input, bind_output};
use crate::instance::Instance;
use crate::netlist::{NetlistMeta, PortDir, PortSpec, SearchableNetlist};
use crate::{Match, Search, State, Wire, WithPath};

#[derive(Debug, Clone)]
pub struct Sdffe<S>
where
    S: State,
{
    pub path: Instance,
    pub clk: Wire<S>,
    pub d: Wire<S>,
    pub reset: Wire<S>,
    pub q: Wire<S>,
}

impl<S> Sdffe<S>
where
    S: State,
{
    /// Uniform constructor so composites can assemble a Search-only shape.
    pub fn new(path: Instance) -> Self {
        Sdffe {
            path: path.clone(),
            clk: crate::Wire::new(path.child("clk".to_string())),
            d: crate::Wire::new(path.child("d".to_string())),
            reset: crate::Wire::new(path.child("reset".to_string())),
            q: crate::Wire::new(path.child("q".to_string())),
        }
    }
}

impl<S> WithPath<S> for Sdffe<S>
where
    S: State,
{
    crate::impl_find_port!(Sdffe, clk, d, reset, q);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

// Static metadata for codegen/introspection (macro-friendly)
impl NetlistMeta for Sdffe<Search> {
    const MODULE_NAME: &'static str = "sdffe";
    const FILE_PATH: &'static str = "examples/patterns/basic/ff/verilog/sdffe.v";

    const PORTS: &'static [PortSpec] = &[
        PortSpec {
            name: "clk",
            dir: PortDir::In,
        },
        PortSpec {
            name: "d",
            dir: PortDir::In,
        },
        PortSpec {
            name: "reset",
            dir: PortDir::In,
        },
        PortSpec {
            name: "q",
            dir: PortDir::Out,
        },
    ];
}

// Query surface; a macro can emit the same impl for any netlist with similar shape
impl SearchableNetlist for Sdffe<Search> {
    type Hit<'p, 'd> = Sdffe<Match<'p, 'd>>;

    fn from_subgraph<'p, 'd>(
        m: &svql_subgraph::SubgraphMatch<'p, 'd>,
        path: Instance,
    ) -> Self::Hit<'p, 'd> {
        // Single-bit ports; multi-bit support could iterate 0..width later
        let clk_m = bind_input(m, "clk", 0);
        let d_m = bind_input(m, "d", 0);
        let reset_m = bind_input(m, "reset", 0);
        let q_m = bind_output(m, "q", 0);

        Sdffe {
            path: path.clone(),
            clk: Wire::with_val(path.child("clk".to_string()), clk_m),
            d: Wire::with_val(path.child("d".to_string()), d_m),
            reset: Wire::with_val(path.child("reset".to_string()), reset_m),
            q: Wire::with_val(path.child("q".to_string()), q_m),
        }
    }
}
