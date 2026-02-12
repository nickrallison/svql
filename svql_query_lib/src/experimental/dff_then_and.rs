//! Experimental pattern matching sequential-to-combinational flow.

use crate::primitives::{AndGate, Sdffe};
use svql_query::prelude::*;

#[derive(Debug, Clone, Composite)]
#[connection(from = ["sdffe", "q"], to = ["and_gate", "a"])]
#[connection(from = ["sdffe", "q"], to = ["and_gate", "b"])]
/// A pattern matching a flip-flop driving an AND gate.
pub struct SdffeThenAnd {
    /// The driving flip-flop.
    #[submodule]
    pub sdffe: Sdffe,
    /// The driven logic gate.
    #[submodule]
    pub and_gate: AndGate,
}
