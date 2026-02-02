use svql_query::prelude::*;

use crate::{AndGate, NotGate, primitives::rec::RecOr};

/// Represents the unlock/bypass logic pattern in CWE1234:
/// - Top-level AND gate (write enable)
/// - Recursive OR tree (bypass conditions)
/// - NOT gate somewhere in the OR tree (negated lock signal)
///
#[derive(Debug, Clone, Composite)]
// function pointer
#[filter(check_fanin_has_not_gates)]
// or possibly a closure
#[filter(|_row, _ctx| { 
    true
})]
pub struct UnlockLogic {
    #[submodule]
    pub top_and: AndGate,
    #[submodule]
    pub rec_or: RecOr,
    #[submodule]
    pub not_gate: NotGate,
    #[alias(output, target = ["top_and", "y"])]
    pub unlock: Wire,
}

fn check_fanin_has_not_gates(
    _row: &svql_query::session::Row<UnlockLogic>,
    _ctx: &svql_query::session::ExecutionContext,
) -> bool {
    todo!("Implement Later")
}
