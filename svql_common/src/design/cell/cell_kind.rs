use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;

use prjunnamed_netlist::Cell;

/// Categorizes netlist primitives into known types for matching.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CellKind {
    /// Buffer / Identity gate.
    Buf,
    /// Inverter gate.
    Not,
    /// Logical AND gate.
    And,
    /// Logical OR gate.
    Or,
    /// Logical XOR gate.
    Xor,
    /// Multiplexer.
    Mux,
    /// Adder/Carry logic.
    Adc,
    /// And-Inverter Graph node.
    Aig,
    /// Equality comparator.
    Eq,
    /// Unsigned Less-Than comparator.
    ULt,
    /// Signed Less-Than comparator.
    SLt,
    /// Shift left.
    Shl,
    /// Unsigned shift right.
    UShr,
    /// Signed shift right.
    SShr,
    /// Logical shift right.
    XShr,
    /// Multiplier.
    Mul,
    /// Unsigned division.
    UDiv,
    /// Unsigned modulo.
    UMod,
    /// Signed division (truncating).
    SDivTrunc,
    /// Signed division (flooring).
    SDivFloor,
    /// Signed modulo (truncating).
    SModTrunc,
    /// Signed modulo (flooring).
    SModFloor,
    /// Match/Case logic.
    Match,
    /// Assignment / Connection.
    Assign,
    /// D-Latch with Set/Reset.
    DLatchSr,
    /// Asynchronous D-Latch.
    ADLatch,
    /// Flip-Flop (Sequential).
    Dff,
    /// Memory/Array block.
    Memory,
    /// IO Buffer.
    IoBuf,
    /// Simulation/Synthesis Target.
    Target,
    /// Other unmapped cell type.
    Other,
    /// Module input port.
    Input,
    /// Module output port.
    Output,
    /// Name/Metadata node.
    Name,
    /// Debugging/Attribute node.
    Debug,
}

impl CellKind {
    /// Returns true if the cell represents a combinational or sequential logic gate.
    #[must_use]
    pub const fn is_logic_gate(&self) -> bool {
        matches!(
            self,
            Self::Buf
                | Self::Not
                | Self::And
                | Self::Or
                | Self::Xor
                | Self::Mux
                | Self::Adc
                | Self::Aig
                | Self::Eq
                | Self::ULt
                | Self::SLt
                | Self::Shl
                | Self::UShr
                | Self::SShr
                | Self::XShr
                | Self::Mul
                | Self::UDiv
                | Self::UMod
                | Self::SDivTrunc
                | Self::SDivFloor
                | Self::SModTrunc
                | Self::SModFloor
                | Self::DLatchSr
                | Self::ADLatch
                | Self::Dff
        )
    }

    /// Returns true if the cell is an input port.
    #[must_use]
    pub const fn is_input(&self) -> bool {
        matches!(self, Self::Input)
    }

    /// Returns true if the cell is an output port.
    #[must_use]
    pub const fn is_output(&self) -> bool {
        matches!(self, Self::Output)
    }

    /// Returns true if the inputs to this cell can be swapped without changing logic.
    #[must_use]
    pub const fn has_commutative_inputs(&self) -> bool {
        matches!(self, Self::And | Self::Or | Self::Xor | Self::Aig)
    }
}

impl From<&Cell> for CellKind {
    fn from(cell: &Cell) -> Self {
        match cell {
            Cell::Buf(..) => Self::Buf,
            Cell::Not(..) => Self::Not,
            Cell::And(..) => Self::And,
            Cell::Or(..) => Self::Or,
            Cell::Xor(..) => Self::Xor,
            Cell::Mux(..) => Self::Mux,
            Cell::Adc(..) => Self::Adc,
            Cell::Aig(..) => Self::Aig,
            Cell::Eq(..) => Self::Eq,
            Cell::ULt(..) => Self::ULt,
            Cell::SLt(..) => Self::SLt,
            Cell::Shl(..) => Self::Shl,
            Cell::UShr(..) => Self::UShr,
            Cell::SShr(..) => Self::SShr,
            Cell::XShr(..) => Self::XShr,
            Cell::Mul(..) => Self::Mul,
            Cell::UDiv(..) => Self::UDiv,
            Cell::UMod(..) => Self::UMod,
            Cell::SDivTrunc(..) => Self::SDivTrunc,
            Cell::SDivFloor(..) => Self::SDivFloor,
            Cell::SModTrunc(..) => Self::SModTrunc,
            Cell::SModFloor(..) => Self::SModFloor,
            Cell::Match(..) => Self::Match,
            Cell::Assign(..) => Self::Assign,
            Cell::Dff(..) => Self::Dff,
            Cell::Memory(..) => Self::Memory,
            Cell::IoBuf(..) => Self::IoBuf,
            Cell::Target(..) => Self::Target,
            Cell::Other(..) => Self::Other,
            Cell::Input(..) => Self::Input,
            Cell::Output(..) => Self::Output,
            Cell::Name(..) => Self::Name,
            Cell::Debug(..) => Self::Debug,
        }
    }
}

impl fmt::Display for CellKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
