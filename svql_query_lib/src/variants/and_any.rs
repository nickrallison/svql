use crate::{State, Wire, primitives::gates::AndGate};
use svql_macros::{netlist, variant};

#[netlist(
    file = "examples/patterns/basic/and/verilog/and_mux.v",
    name = "and_mux"
)]
pub struct AndMux<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

#[netlist(
    file = "examples/patterns/basic/and/verilog/and_nor.v",
    name = "and_nor"
)]
pub struct AndNor<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

#[variant(ports(a, b, y))]
pub enum AndAny<S: State> {
    #[variant(map(a = "a", b = "b", y = "y"))]
    Gate(AndGate<S>),

    #[variant(map(a = "a", b = "b", y = "y"))]
    Mux(AndMux<S>),

    #[variant(map(a = "a", b = "b", y = "y"))]
    Nor(AndNor<S>),
}
