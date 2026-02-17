use crate::HashSet;
use crate::design::cell::PhysicalCellId;
use prjunnamed_netlist::{Net, Trit, Value};
use std::fmt;
use std::ops::RangeBounds;

/// A bundle of one or more nets representing a hardware signal.
/// This type does not carry direction information; use `Port` for that.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire(pub Value);

impl Wire {
    /// Creates a new wire from a value.
    #[must_use]
    pub const fn new(val: Value) -> Self {
        Self(val)
    }

    /// Creates a single-bit wire from a net.
    #[must_use]
    pub fn single(net: Net) -> Self {
        Self(Value::from(net))
    }

    /// Creates a constant wire.
    #[must_use]
    pub fn constant(trit: Trit) -> Self {
        Self(Value::from(trit))
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        Self(self.0.slice(range))
    }

    pub fn concat(&self, other: &Self) -> Self {
        Self(self.0.concat(other.0.clone()))
    }

    /// Returns true if the wire consists entirely of constant bits (0, 1, or X).
    #[must_use]
    pub fn is_constant(&self) -> bool {
        self.0.iter().all(|n| n.is_const())
    }

    /// Returns true if any net in this wire is driven by the other wire.
    #[must_use]
    pub fn drives(&self, other: &Self) -> bool {
        let other_nets: HashSet<Net> = other.0.iter().collect();
        self.0.iter().any(|n| other_nets.contains(&n))
    }

    /// Returns the number of bits in the wire.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the wire has no bits.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the individual nets.
    pub fn iter(&self) -> impl Iterator<Item = Net> + '_ {
        self.0.iter()
    }

    /// Helper to get the first net as a cell ID.
    #[must_use]
    pub fn cell_id(&self) -> Option<PhysicalCellId> {
        self.0.iter().next().and_then(|n| {
            n.as_cell_index()
                .ok()
                .map(|idx| PhysicalCellId::new(idx as u32))
        })
    }

    /// Access the underlying Value
    pub const fn value(&self) -> &Value {
        &self.0
    }
}

impl FromIterator<Net> for Wire {
    fn from_iter<T: IntoIterator<Item = Net>>(iter: T) -> Self {
        Self(Value::from_iter(iter))
    }
}

impl IntoIterator for Wire {
    type Item = Net;
    type IntoIter = <prjunnamed_netlist::Value as std::iter::IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<PhysicalCellId> for Wire {
    fn from(cell_id: PhysicalCellId) -> Self {
        Self::single(Net::from_cell_index(cell_id.storage_key() as usize))
    }
}

impl From<Net> for Wire {
    fn from(net: Net) -> Self {
        Self::single(net)
    }
}

impl fmt::Display for Wire {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A wire with an associated direction (Input, Output, Inout).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Port {
    /// The underlying wire value.
    pub wire: Wire,
    /// The direction of the port.
    pub direction: PortDirection,
}

impl Port {
    /// Creates a new port.
    #[must_use]
    pub const fn new(wire: Wire, direction: PortDirection) -> Self {
        Self { wire, direction }
    }
}

/// Defines the direction of a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDirection {
    /// Internal signal.
    None,
    /// Input port.
    Input,
    /// Output port.
    Output,
    /// Bidirectional port.
    Inout,
}

#[derive(Debug, Clone, Copy)]
pub struct PortDecl {
    pub name: &'static str,
    pub direction: PortDirection,
}

impl PortDecl {
    pub const fn input(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Input,
        }
    }
    pub const fn output(name: &'static str) -> Self {
        Self {
            name,
            direction: PortDirection::Output,
        }
    }
}

impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::None => "None",
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Inout => "Inout",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, quickcheck};

    #[derive(Debug, Clone)]
    struct ArbitraryNet(Net);

    // Helper to generate arbitrary Nets for testing
    impl Arbitrary for ArbitraryNet {
        fn arbitrary(g: &mut Gen) -> Self {
            let variant = u8::arbitrary(g) % 3;
            match variant {
                0 => Self(Net::ZERO),
                1 => Self(Net::ONE),
                _ => Self(Net::from_cell_index(u32::arbitrary(g) as usize % 1000)),
            }
        }
    }

    quickcheck! {
        fn prop_wire_len_consistency(nets: Vec<ArbitraryNet>) -> bool {
            let nets: Vec<Net> = nets.into_iter().map(|an| an.0).collect();
            let wire = Wire::new(Value::from(nets.clone()));
            wire.len() == nets.len()
        }

        fn prop_wire_is_empty(nets: Vec<ArbitraryNet>) -> bool {
            let nets: Vec<Net> = nets.into_iter().map(|an| an.0).collect();
            let wire = Wire::new(Value::from(nets.clone()));
            wire.is_empty() == nets.is_empty()
        }

        fn prop_wire_drives_self(nets: Vec<ArbitraryNet>) -> bool {
            let nets: Vec<Net> = nets.into_iter().map(|an| an.0).collect();
            let wire = Wire::new(Value::from(nets));
            // A wire should always drive itself (intersection is non-empty)
            wire.drives(&wire)
        }
    }
}
