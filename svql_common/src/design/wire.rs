//! Wire and net abstractions for hardware connectivity.
//!
//! Provides types for representing physical wires, primary ports,
//! and constants, along with their relative directions (Input/Output).

use crate::*;
use contracts::*;
use std::sync::Arc;

/// A reference to a single net in the design.
/// This is the atomic unit of connectivity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WireRef {
    /// A specific net index (bit).
    Net(u32),
    /// A constant value (0 or 1).
    Constant(bool),
}

impl WireRef {
    /// Check if this is a net reference
    #[must_use]
    pub const fn is_net(&self) -> bool {
        matches!(self, Self::Net(_))
    }

    /// Get the net index if this is a net reference
    #[must_use]
    pub const fn as_net(&self) -> Option<u32> {
        match self {
            Self::Net(idx) => Some(*idx),
            _ => None,
        }
    }
}

impl std::fmt::Display for WireRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Net(idx) => write!(f, "Net({})", idx),
            Self::Constant(value) => write!(f, "{}", if *value { "1'b1" } else { "1'b0" }),
        }
    }
}

/// A signal in the design, composed of one or more WireRefs (nets).
/// This represents a bus or a single-bit wire.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire {
    /// The nets making up this wire.
    nets: Vec<WireRef>,
    /// Direction relative to the pattern port.
    direction: PortDirection,
}

impl std::fmt::Display for Wire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.nets.is_empty() {
            return write!(f, "empty");
        }

        if self.nets.len() == 1 {
            return write!(f, "{}", self.nets[0]);
        }

        write!(f, "{{")?;
        for (i, r) in self.nets.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", r)?;
        }
        write!(f, "}}")
    }
}

impl Wire {
    pub fn new(nets: Vec<WireRef>, direction: PortDirection) -> Self {
        Self { nets, direction }
    }

    pub fn single(net: u32, direction: PortDirection) -> Self {
        Self {
            nets: vec![WireRef::Net(net)],
            direction,
        }
    }

    pub fn constant(value: bool, direction: PortDirection) -> Self {
        Self {
            nets: vec![WireRef::Constant(value)],
            direction,
        }
    }

    pub fn direction(&self) -> PortDirection {
        self.direction
    }

    pub fn nets(&self) -> &[WireRef] {
        &self.nets
    }

    pub fn is_empty(&self) -> bool {
        self.nets.is_empty()
    }

    // Helper to check if this wire drives another wire (intersection of nets)
    pub fn drives(&self, other: &Wire) -> bool {
        self.nets.iter().any(|n| other.nets.contains(n))
    }

    /// Check if this is a constant
    #[must_use]
    pub fn is_constant(&self) -> bool {
        self.nets.iter().all(|n| matches!(n, WireRef::Constant(_)))
    }

    /// Helper to get the first net as a cell ID (for compatibility with 1-bit primitives)
    #[must_use]
    pub fn cell_id(&self) -> Option<PhysicalCellId> {
        self.nets()
            .first()
            .and_then(|n| n.as_net())
            .map(PhysicalCellId::new)
    }
}

/// Defines the direction of a port column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDirection {
    /// Not a port (internal wire, submodule reference, or metadata).
    None,
    /// Input port (receives signal).
    Input,
    /// Output port (drives signal).
    Output,
    /// Bidirectional port.
    Inout,
}

impl std::fmt::Display for PortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Input => write!(f, "Input"),
            Self::Output => write!(f, "Output"),
            Self::Inout => write!(f, "Inout"),
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, quickcheck};
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    struct ArbitraryWireRef(WireRef);

    impl Arbitrary for ArbitraryWireRef {
        fn arbitrary(g: &mut Gen) -> Self {
            let variant = u8::arbitrary(g) % 2;
            let wire = match variant {
                0 => WireRef::Net(u32::arbitrary(g)),
                _ => WireRef::Constant(bool::arbitrary(g)),
            };
            Self(wire)
        }
    }

    #[derive(Clone, Debug, Copy)]
    struct ArbitraryPortDirection(PortDirection);

    impl Arbitrary for ArbitraryPortDirection {
        fn arbitrary(g: &mut Gen) -> Self {
            let variants = [
                PortDirection::None,
                PortDirection::Input,
                PortDirection::Output,
                PortDirection::Inout,
            ];
            Self(*g.choose(&variants).unwrap())
        }
    }

    quickcheck! {
        fn prop_wire_direction_preserved(wr: ArbitraryWireRef, dir: ArbitraryPortDirection) -> bool {
            let wire = Wire::new(vec![wr.0.clone()], dir.0);
            wire.direction() == dir.0
        }

        fn prop_wire_net_consistency(wr: ArbitraryWireRef) -> bool {
            let is_net = matches!(wr.0, WireRef::Net(_));
            let wire = Wire::new(vec![wr.0.clone()], PortDirection::None);
            wire.nets().first().map(|n| n.is_net()).unwrap_or(false) == is_net
        }
    }
}
