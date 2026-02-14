//! Combinational logic patterns for bypass/unlock signals.

use svql_query::prelude::*;

use crate::{AndGate, NotGate, primitives::rec::RecOr};

/// Represents the unlock/bypass logic pattern in CWE1234:
/// - Top-level AND gate (write enable)
/// - Recursive OR tree (bypass conditions)
/// - NOT gate somewhere in the OR tree (negated lock signal)
///
/// Uses `#[connection(kind = "any")]` to check if the NOT gate output is one of the OR tree's leaf inputs.
///
#[derive(Debug, Clone, Composite)]
#[or_to(from = ["rec_or", "y"], to = [["top_and", "a"], ["top_and", "b"]])]
#[connection(from = ["not_gate", "y"], to = ["rec_or", "leaf_inputs"], kind = "any")]
pub struct UnlockLogic {
    /// Logical AND gate combining various lock signals.
    #[submodule]
    pub top_and: AndGate,
    /// Discovered tree of OR gates checking for unlock conditions.
    #[submodule]
    pub rec_or: RecOr,
    /// The specific inverter creating the bypass vulnerability.
    #[submodule]
    pub not_gate: NotGate,
    /// The resulting unlock port wire.
    #[alias(output, target = ["top_and", "y"])]
    pub unlock: Wire,
}
