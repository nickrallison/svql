use crate::prelude::*;

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
    #[must_use] 
    pub const fn new(id: CellId, direction: PortDirection) -> Self {
        match direction {
            PortDirection::Input => Self::Input(id),
            PortDirection::Output => Self::Output(id),
            PortDirection::Inout => Self::Inout(id),
            PortDirection::None => Self::Unknown(id),
        }
    }

    /// Create from raw u64 (for backward compatibility).
    #[must_use] 
    pub fn from_u64(id: u64, direction: PortDirection) -> Self {
        Self::new(CellId::from_u64(id), direction)
    }

    /// Get the underlying cell ID.
    #[must_use] 
    pub const fn id(&self) -> CellId {
        match self {
            Self::Input(id) => *id,
            Self::Output(id) => *id,
            Self::Inout(id) => *id,
            Self::Unknown(id) => *id,
        }
    }

    /// Get the direction.
    #[must_use] 
    pub const fn direction(&self) -> PortDirection {
        match self {
            Self::Input(_) => PortDirection::Input,
            Self::Output(_) => PortDirection::Output,
            Self::Inout(_) => PortDirection::Inout,
            Self::Unknown(_) => PortDirection::None,
        }
    }

    /// Get the raw u64 value (for backward compatibility).
    #[must_use] 
    pub const fn as_u64(&self) -> u64 {
        self.id().as_u64()
    }
}
