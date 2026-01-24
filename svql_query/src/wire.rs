use crate::session::PortDirection;

/// A wire reference containing an ID and port direction.
/// Replaces the concept of a raw CellId with a directed handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wire {
    Input(u64),
    Output(u64),
    /// For bidirectional ports or unknown direction wires
    Inout(u64),
    /// For internal wires or cases where direction doesn't apply
    Unknown(u64),
}

impl Wire {
    /// Create a new Wire reference.
    pub fn new(id: u64, direction: PortDirection) -> Self {
        match direction {
            PortDirection::Input => Wire::Input(id),
            PortDirection::Output => Wire::Output(id),
            PortDirection::Inout => Wire::Inout(id),
            PortDirection::None => Wire::Unknown(id),
        }
    }

    /// Get the underlying cell ID.
    pub fn id(&self) -> u64 {
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
}
