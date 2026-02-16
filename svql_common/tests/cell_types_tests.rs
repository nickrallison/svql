#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

use prjunnamed_netlist::Cell;
use svql_common::{CellKind, GraphNodeIdx, PhysicalCellId};

mod common;

#[test]
fn test_physical_cell_id_creation() {
    let id = PhysicalCellId::new(42);
    assert_eq!(id.storage_key(), 42);
}

#[test]
fn test_physical_cell_id_display() {
    let id = PhysicalCellId::new(42);
    assert_eq!(format!("{}", id), "p42");
}

#[test]
fn test_physical_cell_id_equality() {
    let id1 = PhysicalCellId::new(42);
    let id2 = PhysicalCellId::new(42);
    let id3 = PhysicalCellId::new(43);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_graph_node_idx_creation() {
    let idx = GraphNodeIdx::new(42);
    assert_eq!(idx.as_usize(), 42);
}

#[test]
fn test_graph_node_idx_conversions() {
    let idx: GraphNodeIdx = 42usize.into();
    assert_eq!(idx.as_usize(), 42);

    let back: usize = idx.into();
    assert_eq!(back, 42);
}

#[test]
fn test_graph_node_idx_display() {
    let idx = GraphNodeIdx::new(42);
    assert_eq!(format!("{}", idx), "n42");
}

#[test]
fn test_cell_kind_from_cell() {
    // Test all cell kinds
    let cell = Cell::And(
        prjunnamed_netlist::Value::default(),
        prjunnamed_netlist::Value::default(),
    );
    assert_eq!(CellKind::from(&cell), CellKind::And);

    let cell = Cell::Not(prjunnamed_netlist::Value::default());
    assert_eq!(CellKind::from(&cell), CellKind::Not);

    let cell = Cell::Input("clk".to_string(), 1);
    assert_eq!(CellKind::from(&cell), CellKind::Input);
}

#[test]
fn test_cell_kind_is_logic_gate() {
    assert!(CellKind::And.is_logic_gate());
    assert!(CellKind::Or.is_logic_gate());
    assert!(CellKind::Not.is_logic_gate());
    assert!(CellKind::Dff.is_logic_gate());

    assert!(!CellKind::Input.is_logic_gate());
    assert!(!CellKind::Output.is_logic_gate());
    assert!(!CellKind::Memory.is_logic_gate());
}

#[test]
fn test_cell_kind_has_commutative_inputs() {
    assert!(CellKind::And.has_commutative_inputs());
    assert!(CellKind::Or.has_commutative_inputs());
    assert!(CellKind::Xor.has_commutative_inputs());

    assert!(!CellKind::Not.has_commutative_inputs());
    assert!(!CellKind::Mux.has_commutative_inputs());
}
