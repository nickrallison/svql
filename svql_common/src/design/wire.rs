use crate::*;
use std::sync::Arc;

/// Core wire reference type that can be stored in tables.
///
/// This enum can represent a wire driven by a cell output, a primary input/output port,
/// or a constant value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WireRef {
    /// Reference via the stable physical ID
    Cell(PhysicalCellId),
    /// Primary input/output port (module boundary)
    PrimaryPort(Arc<str>),
    /// Constant value (0 or 1)
    Constant(bool),
}

impl WireRef {
    /// Convert to a contextual Wire with direction information
    #[must_use]
    pub fn to_wire(self, direction: PortDirection) -> Wire {
        match self {
            Self::Cell(id) => Wire::Cell { id, direction },
            Self::PrimaryPort(name) => Wire::PrimaryPort { name, direction },
            Self::Constant(value) => Wire::Constant { value },
        }
    }

    /// Check if this is a cell reference
    #[must_use]
    pub const fn is_cell(&self) -> bool {
        matches!(self, Self::Cell(_))
    }

    /// Get the cell ID if this is a cell reference
    #[must_use]
    pub const fn as_cell(&self) -> Option<PhysicalCellId> {
        match self {
            Self::Cell(id) => Some(*id),
            _ => None,
        }
    }
}

/// A wire with contextual direction information.
///
/// This is the wire type exposed to queries. It wraps a `WireRef` with
/// additional direction metadata from the schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Wire {
    /// Cell-driven wire
    Cell {
        id: PhysicalCellId,
        direction: PortDirection,
    },
    /// Primary port at module boundary
    PrimaryPort {
        name: Arc<str>,
        direction: PortDirection,
    },
    /// Constant value
    Constant { value: bool },
}

impl Wire {
    /// Create a Wire from a cell ID and direction
    #[must_use]
    pub const fn new(id: PhysicalCellId, direction: PortDirection) -> Self {
        Self::Cell { id, direction }
    }

    /// Create a Wire from a `WireRef` and direction
    #[must_use]
    pub fn from_ref(wire_ref: WireRef, direction: PortDirection) -> Self {
        wire_ref.to_wire(direction)
    }

    /// Get the cell ID if this is a cell-driven wire
    #[must_use]
    pub const fn cell_id(&self) -> Option<PhysicalCellId> {
        match self {
            Self::Cell { id, .. } => Some(*id),
            _ => None,
        }
    }

    /// Get the direction (None for constants)
    #[must_use]
    pub const fn direction(&self) -> Option<PortDirection> {
        match self {
            Self::Cell { direction, .. } | Self::PrimaryPort { direction, .. } => Some(*direction),
            Self::Constant { .. } => None,
        }
    }

    /// Check if this is a constant
    #[must_use]
    pub const fn is_constant(&self) -> bool {
        matches!(self, Self::Constant { .. })
    }

    /// Check if this is a primary port
    #[must_use]
    pub const fn is_primary_port(&self) -> bool {
        matches!(self, Self::PrimaryPort { .. })
    }

    /// Check if this is a cell
    #[must_use]
    pub const fn is_cell(&self) -> bool {
        matches!(self, Self::Cell { .. })
    }

    /// Legacy compatibility: get underlying cell ID (panics if not a cell)
    /// Use `cell_id()` for safe access.
    #[must_use]
    #[deprecated(note = "Use cell_id() instead")]
    pub fn id(&self) -> PhysicalCellId {
        self.cell_id().expect("Wire is not a cell reference")
    }

    /// Get the raw u64 value (for backward compatibility).
    /// Panics if not a cell reference.
    #[must_use]
    #[deprecated(note = "Use cell_id() instead")]
    #[allow(deprecated)]
    pub fn as_u64(&self) -> u64 {
        u64::from(self.id())
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
