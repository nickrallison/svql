use crate::primitives::{AndGate, Sdffe};
use svql_query::prelude::*;

#[derive(Debug, Clone, Composite)]
#[connection(from = ["sdffe", "q"], to = ["and_gate", "a"])]
#[connection(from = ["sdffe", "q"], to = ["and_gate", "b"])]
pub struct SdffeThenAnd {
    #[submodule]
    pub sdffe: Sdffe,
    #[submodule]
    pub and_gate: AndGate,
}
