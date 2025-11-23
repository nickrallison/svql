use crate::primitives::and::AndGate;
use crate::primitives::and::AndMux;
use crate::primitives::and::AndNor;
use svql_macros::variant;

variant! {
    name: AndAny,
    variants: [
        (Gate, "and_gate", AndGate),
        (Mux, "and_mux",  AndMux),
        (Nor, "and_nor",  AndNor)
    ]
}
