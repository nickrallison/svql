use crate::primitives::and::AndGate;
use crate::primitives::and::AndMux;
use crate::primitives::and::AndNor;
use svql_macros::enum_composite;

enum_composite! {
    name: AndAny,
    variants: [
        (Gate, "and_gate", AndGate),
        (Mux, "and_mux",  AndMux),
        (Nor, "and_nor",  AndNor)
    ]
}
