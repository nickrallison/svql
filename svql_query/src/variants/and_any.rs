use crate::State;
use crate::primitives::and::{AndGate, AndMux, AndNor};
use svql_macros::variant;

#[variant(ports(a, b, y))]
pub enum AndAny<S: State> {
    #[variant(map(a = "a", b = "b", y = "y"))]
    Gate(AndGate<S>),

    #[variant(map(a = "a", b = "b", y = "y"))]
    Mux(AndMux<S>),

    #[variant(map(a = "a", b = "b", y = "y"))]
    Nor(AndNor<S>),
}
