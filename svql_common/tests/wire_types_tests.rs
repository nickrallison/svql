#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

use std::sync::Arc;
use svql_common::{PhysicalCellId, PortDirection, Wire, WireRef};

mod common;

#[test]
fn test_wire_ref_net() {
    let wire_ref = WireRef::Net(42);

    assert!(wire_ref.is_net());
    assert_eq!(wire_ref.as_net(), Some(42));
}

#[test]
fn test_wire_ref_constant() {
    let wire_ref = WireRef::Constant(true);

    assert!(!wire_ref.is_net());
    assert!(wire_ref.as_net().is_none());
}

#[test]
fn test_wire_creation() {
    let wire = Wire::single(42, PortDirection::Output);

    assert_eq!(wire.cell_id(), Some(PhysicalCellId::new(42)));
    assert_eq!(wire.direction(), PortDirection::Output);
    assert!(!wire.is_constant());
}

#[test]
fn test_wire_constant() {
    let wire = Wire::constant(true, PortDirection::None);

    assert!(wire.is_constant());
    assert_eq!(wire.direction(), PortDirection::None);
    assert!(wire.cell_id().is_none());
}

#[test]
fn test_port_direction_display() {
    assert_eq!(format!("{}", PortDirection::Input), "Input");
    assert_eq!(format!("{}", PortDirection::Output), "Output");
    assert_eq!(format!("{}", PortDirection::Inout), "Inout");
    assert_eq!(format!("{}", PortDirection::None), "None");
}
