//! Cell definitions, wrappers, and type-safe identifiers for SVQL.
//!
//! This module provides the core cell types used across the SVQL project:
//! - [`PhysicalCellId`]: Persistent ID from the netlist source (e.g., debug_index from prjunnamed).
//! - [`GraphNodeIdx`]: Local identifier within a specific GraphIndex array.
//! - [`CellKind`]: Categorization of netlist primitives.
//! - [`CellWrapper`]: A wrapper around a netlist cell with stable identity.

/// Categorization of netlist primitives.
mod cell_kind;
/// Stable identity wrapper for netlist cells.
mod cell_wrapper;
/// Local graph indexing types.
mod graph_node_id;
/// Persistent netlist identifier types.
mod physical_cell_id;

pub use cell_kind::CellKind;
pub use cell_wrapper::CellWrapper;
pub use graph_node_id::GraphNodeIdx;
pub use physical_cell_id::PhysicalCellId;

#[cfg(test)]
mod tests {
    use super::*;
    use prjunnamed_netlist::Cell;

    #[test]
    fn test_graph_node_idx_creation() {
        let id = GraphNodeIdx::new(42);
        assert_eq!(id.as_usize(), 42);
    }

    #[test]
    fn test_graph_node_idx_display() {
        let id = GraphNodeIdx::new(42);
        assert_eq!(format!("{}", id), "n42");
    }

    #[test]
    fn test_graph_node_idx_ordering() {
        let id1 = GraphNodeIdx::new(1);
        let id2 = GraphNodeIdx::new(2);
        assert!(id1 < id2);
        assert!(id2 > id1);
    }

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
    fn test_graph_node_idx_conversions() {
        let idx: GraphNodeIdx = 42usize.into();
        assert_eq!(idx.as_usize(), 42);

        let back: usize = idx.into();
        assert_eq!(back, 42);
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
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, quickcheck};

    // -- PhysicalCellId --

    #[derive(Clone, Debug)]
    struct ArbitraryPhysicalCellId(PhysicalCellId);

    impl Arbitrary for ArbitraryPhysicalCellId {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(PhysicalCellId::new(u32::arbitrary(g)))
        }
    }

    quickcheck! {
        fn prop_physical_id_roundtrip(id: ArbitraryPhysicalCellId) -> bool {
            let raw = id.0.storage_key();
            PhysicalCellId::new(raw) == id.0
        }

        fn prop_physical_id_ordering(a: ArbitraryPhysicalCellId, b: ArbitraryPhysicalCellId) -> bool {
            a.0.cmp(&b.0) == a.0.storage_key().cmp(&b.0.storage_key())
        }
    }

    // -- GraphNodeIdx --

    #[derive(Clone, Debug)]
    struct ArbitraryGraphNodeIdx(GraphNodeIdx);

    impl Arbitrary for ArbitraryGraphNodeIdx {
        fn arbitrary(g: &mut Gen) -> Self {
            // Limit size to avoid overflow in usize conversion on 32-bit targets if necessary
            let val = u32::arbitrary(g) % 100_000;
            Self(GraphNodeIdx::new(val))
        }
    }

    quickcheck! {
        fn prop_graph_idx_roundtrip(idx: ArbitraryGraphNodeIdx) -> bool {
            let raw: u32 = idx.0.into();
            GraphNodeIdx::new(raw) == idx.0
        }

        fn prop_graph_idx_usize_conversion(idx: ArbitraryGraphNodeIdx) -> bool {
            let as_usize: usize = idx.0.as_usize();
            GraphNodeIdx::from(as_usize) == idx.0
        }
    }
}
