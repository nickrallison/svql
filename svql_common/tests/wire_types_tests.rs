#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

use std::sync::Arc;
use svql_common::{PhysicalCellId, PortDirection, Wire, WireRef};

mod common;

#[test]
fn test_wire_ref_cell() {
    let id = PhysicalCellId::new(42);
    let wire_ref = WireRef::Cell(id);

    assert!(wire_ref.is_cell());
    assert_eq!(wire_ref.as_cell(), Some(id));
}

#[test]
fn test_wire_ref_primary_port() {
    let name = Arc::<str>::from("clk");
    let wire_ref = WireRef::PrimaryPort(name);

    assert!(!wire_ref.is_cell());
    assert!(wire_ref.as_cell().is_none());
}

#[test]
fn test_wire_ref_constant() {
    let wire_ref = WireRef::Constant(true);

    assert!(!wire_ref.is_cell());
    assert!(wire_ref.as_cell().is_none());
}

#[test]
fn test_wire_ref_to_wire() {
    let id = PhysicalCellId::new(42);
    let wire_ref = WireRef::Cell(id);
    let wire = wire_ref.to_wire(PortDirection::Input);

    assert_eq!(wire.direction(), Some(PortDirection::Input));
    assert_eq!(wire.cell_id(), Some(id));
}

#[test]
fn test_wire_creation() {
    let id = PhysicalCellId::new(42);
    let wire = Wire::new(id, PortDirection::Output);

    assert_eq!(wire.cell_id(), Some(id));
    assert_eq!(wire.direction(), Some(PortDirection::Output));
    assert!(wire.is_cell());
    assert!(!wire.is_constant());
    assert!(!wire.is_primary_port());
}

#[test]
fn test_wire_constant() {
    let wire = Wire::Constant { value: true };

    assert!(wire.is_constant());
    assert!(wire.direction().is_none());
    assert!(wire.cell_id().is_none());
}

#[test]
fn test_wire_primary_port() {
    let name = Arc::<str>::from("clk");
    let wire = Wire::PrimaryPort {
        name,
        direction: PortDirection::Input,
    };

    assert!(wire.is_primary_port());
    assert_eq!(wire.direction(), Some(PortDirection::Input));
}

#[test]
fn test_port_direction_display() {
    assert_eq!(format!("{}", PortDirection::Input), "Input");
    assert_eq!(format!("{}", PortDirection::Output), "Output");
    assert_eq!(format!("{}", PortDirection::Inout), "Inout");
    assert_eq!(format!("{}", PortDirection::None), "None");
}
