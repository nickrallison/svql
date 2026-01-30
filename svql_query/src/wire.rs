use crate::cell_id::CellId;
use crate::session::PortDirection;

/// A wire reference containing a cell ID and port direction.
///
/// Replaces the concept of a raw numeric ID with a strongly-typed handle
/// that tracks both the cell identity and its directionality in the design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wire {
    /// Input port wire
    Input(CellId),
    /// Output port wire
    Output(CellId),
    /// Bidirectional port wire
    Inout(CellId),
    /// Internal wire or unknown direction
    Unknown(CellId),
}

impl Wire {
    /// Create a new Wire reference from a cell ID and direction.
    pub fn new(id: CellId, direction: PortDirection) -> Self {
        match direction {
            PortDirection::Input => Wire::Input(id),
            PortDirection::Output => Wire::Output(id),
            PortDirection::Inout => Wire::Inout(id),
            PortDirection::None => Wire::Unknown(id),
        }
    }

    /// Create from raw u64 (for backward compatibility).
    pub fn from_u64(id: u64, direction: PortDirection) -> Self {
        Self::new(CellId::from_u64(id), direction)
    }

    /// Get the underlying cell ID.
    pub fn id(&self) -> CellId {
        match self {
            Wire::Input(id) => *id,
            Wire::Output(id) => *id,
            Wire::Inout(id) => *id,
            Wire::Unknown(id) => *id,
        }
    }

    /// Get the direction.
    pub fn direction(&self) -> PortDirection {
        match self {
            Wire::Input(_) => PortDirection::Input,
            Wire::Output(_) => PortDirection::Output,
            Wire::Inout(_) => PortDirection::Inout,
            Wire::Unknown(_) => PortDirection::None,
        }
    }

    /// Get the raw u64 value (for backward compatibility).
    pub fn as_u64(&self) -> u64 {
        self.id().as_u64()
    }
}
