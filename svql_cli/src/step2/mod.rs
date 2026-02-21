use svql_query::prelude::*;

// Task: Define the HalfAdder struct with #[derive(Netlist)]

#[derive(Debug, Clone, Netlist)]
#[netlist(file = "examples/fixtures/logic/half_adder.v", module = "half_adder")]
pub struct HalfAdder {
    #[port(input)]
    pub a: Wire,
    #[port(input)]
    pub b: Wire,
    #[port(output)]
    pub sum: Wire,
    #[port(output)]
    pub carry: Wire,
}
