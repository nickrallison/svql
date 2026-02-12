//! Combinational logic patterns for bypass/unlock signals.

use svql_query::prelude::*;

use crate::{AndGate, NotGate, OrGate, primitives::rec::RecOr};

/// Represents the unlock/bypass logic pattern in CWE1234:
/// - Top-level AND gate (write enable)
/// - Recursive OR tree (bypass conditions)
/// - NOT gate somewhere in the OR tree (negated lock signal)
///
#[derive(Debug, Clone, Composite)]
#[or_to(from = ["rec_or", "y"], to = [["top_and", "a"], ["top_and", "b"]])]
#[filter(check_fanin_has_not_gates)]
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

/// Validates that the NOT gate output feeds somewhere into the OR tree.
fn check_fanin_has_not_gates(row: &Row<UnlockLogic>, ctx: &ExecutionContext) -> bool {
    let not_output = row.resolve(Selector::static_path(&["not_gate", "y"]), ctx);
    let rec_or_ref: Option<Ref<RecOr>> = row.sub("rec_or");

    let (Some(not_out), Some(rec_or_ref)) = (not_output, rec_or_ref) else {
        return true;
    };

    // Get RecOr table
    let Some(rec_or_any) = ctx.get_any_table(std::any::TypeId::of::<RecOr>()) else {
        return true;
    };
    let Some(rec_or_table) = rec_or_any.as_any().downcast_ref::<Table<RecOr>>() else {
        return true;
    };

    // Get OrGate table (RecOr.base is OrGate, not AndGate!)
    let Some(or_gate_any) = ctx.get_any_table(std::any::TypeId::of::<OrGate>()) else {
        return true;
    };
    let Some(or_gate_table) = or_gate_any.as_any().downcast_ref::<Table<OrGate>>() else {
        return true;
    };

    check_tree_recursive(rec_or_ref, not_out.cell_id().expect("NOT output must be a cell"), rec_or_table, or_gate_table)
}

fn check_tree_recursive(
    rec_or_ref: Ref<RecOr>,
    not_id: PhysicalCellId,
    rec_or_table: &Table<RecOr>,
    or_gate_table: &Table<OrGate>,
) -> bool {
    let Some(rec_or_row) = rec_or_table.row(rec_or_ref.index()) else {
        return false;
    };

    // Get base OrGate
    let Some(base_or_ref) = rec_or_row.sub::<OrGate>("base") else {
        return false;
    };

    let Some(or_row) = or_gate_table.row(base_or_ref.index()) else {
        return false;
    };

    // Check if NOT feeds this OR gate
    if or_row.wire("a").and_then(|w| w.cell_id()) == Some(not_id)
        || or_row.wire("b").and_then(|w| w.cell_id()) == Some(not_id)
    {
        return true;
    }

    // Recursively check children
    if let Some(left) = rec_or_row.sub::<RecOr>("left_child")
        && check_tree_recursive(left, not_id, rec_or_table, or_gate_table)
    {
        return true;
    }

    if let Some(right) = rec_or_row.sub::<RecOr>("right_child")
        && check_tree_recursive(right, not_id, rec_or_table, or_gate_table)
    {
        return true;
    }

    false
}
