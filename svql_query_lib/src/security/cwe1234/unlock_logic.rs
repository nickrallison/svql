use std::{
    cell,
    collections::{HashSet, VecDeque},
};

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

/// Validates that the NOT gate output directly connects to the RecOr base OR gate inputs.
///
/// Checks if `not_gate.y` matches either `rec_or.base.a` or `rec_or.base.b`.
/// This ensures the NOT gate actually feeds into the OR tree rather than being
/// an unrelated component elsewhere in the design.
fn check_fanin_has_not_gates(
    row: &svql_query::session::Row<UnlockLogic>,
    ctx: &svql_query::session::ExecutionContext,
) -> bool {
    // Get the NOT gate output
    let Some(not_output) = row.resolve(Selector::static_path(&["not_gate", "y"]), ctx) else {
        tracing::trace!("Failed to resolve not_gate.y");
        return false;
    };

    // Get the RecOr base OR gate's input A
    let Some(or_input_a) = row.resolve(Selector::static_path(&["rec_or", "base", "a"]), ctx) else {
        tracing::trace!("Failed to resolve rec_or.base.a");
        return false;
    };

    // Get the RecOr base OR gate's input B
    let Some(or_input_b) = row.resolve(Selector::static_path(&["rec_or", "base", "b"]), ctx) else {
        tracing::trace!("Failed to resolve rec_or.base.b");
        return false;
    };

    // Check if NOT output connects to either OR input
    let connects_to_a = not_output.id() == or_input_a.id();
    let connects_to_b = not_output.id() == or_input_b.id();

    let result = connects_to_a || connects_to_b;

    if !result {
        tracing::debug!(
            "NOT gate output {} does not connect to OR inputs ({}, {}) - rejecting",
            not_output.id(),
            or_input_a.id(),
            or_input_b.id()
        );
    }

    result
}
