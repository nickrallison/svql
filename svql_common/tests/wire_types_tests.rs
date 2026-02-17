#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

use prjunnamed_netlist::{Net, Trit};
use svql_common::{PhysicalCellId, Port, PortDirection, Wire};

mod common;

#[test]
fn test_wire_single_net() {
    // Wire::single now expects only a Net object
    let wire = Wire::single(Net::from_cell_index(42));

    assert_eq!(wire.len(), 1);
    assert_eq!(wire.cell_id(), Some(PhysicalCellId::new(42)));
    assert!(!wire.is_constant());
}

#[test]
fn test_wire_constant() {
    // Wire::constant now expects only a Trit
    let wire = Wire::constant(Trit::One);

    assert!(wire.is_constant());
    assert!(wire.cell_id().is_none());
    assert_eq!(wire.len(), 1);
}

#[test]
fn test_port_creation() {
    // Use the new Port type to associate direction with a Wire
    let wire = Wire::single(Net::from_cell_index(10));
    let port = Port::new(wire.clone(), PortDirection::Output);

    assert_eq!(port.wire, wire);
    assert_eq!(port.direction, PortDirection::Output);
}

#[test]
fn test_wire_drives() {
    let net_a = Net::from_cell_index(1);
    let net_b = Net::from_cell_index(2);

    let wire_a = Wire::single(net_a);
    let wire_b = Wire::single(net_b);

    // Same net drives
    assert!(wire_a.drives(&Wire::single(net_a)));

    // Different nets do not drive
    assert!(!wire_a.drives(&wire_b));
}

#[test]
fn test_port_direction_display() {
    assert_eq!(format!("{}", PortDirection::Input), "Input");
    assert_eq!(format!("{}", PortDirection::Output), "Output");
    assert_eq!(format!("{}", PortDirection::Inout), "Inout");
    assert_eq!(format!("{}", PortDirection::None), "None");
}
