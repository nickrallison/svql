use svql_query::prelude::*;

use crate::{AndGate, NotGate, primitives::rec::RecOr};

/// Represents the unlock/bypass logic pattern in CWE1234:
/// - Top-level AND gate (write enable)
/// - Recursive OR tree (bypass conditions)
/// - NOT gate somewhere in the OR tree (negated lock signal)
///
#[derive(Debug, Clone, Composite)]
#[or_to(from = ["rec_or", "y"], to = [["top_and", "a"], ["top_and", "b"]])]
#[filter(check_fanin_has_not_gates)]
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

/// Validates that the NOT gate output directly feeds into the OR tree.
fn check_fanin_has_not_gates(
    row: &svql_query::session::Row<UnlockLogic>,
    ctx: &svql_query::session::ExecutionContext,
) -> bool {
    // Resolve the NOT gate output
    let Some(not_output) = row.resolve(Selector::static_path(&["not_gate", "y"]), ctx) else {
        return false;
    };

    // Resolve the base OR gate inputs within the recursive tree
    let Some(or_input_a) = row.resolve(Selector::static_path(&["rec_or", "base", "a"]), ctx) else {
        return false;
    };

    let Some(or_input_b) = row.resolve(Selector::static_path(&["rec_or", "base", "b"]), ctx) else {
        return false;
    };

    // The NOT gate must be one of the inputs to the root of the OR tree
    not_output.id() == or_input_a.id() || not_output.id() == or_input_b.id()
}
