use crate::{AndGate, NotGate, security::dff_enable::DffEnable};
use svql_query::prelude::*;

/// Composite pattern representing a register with an access lock, which may be implemented using a DFF with enable and associated logic gates.
#[derive(Debug, Clone, Composite)]
#[or_to(from = ["not", "y"], to = [["and", "a"], ["and", "b"]])]
#[connection(from = ["and", "y"], to = ["dff_enable", "write_en"])]
pub struct LockedRegister {
    /// DFF with enable, which may be implemented as a primitive or via mux logic.
    #[submodule]
    pub dff_enable: DffEnable,

    /// AND gate to combine the lock signal with the data input.
    #[submodule]
    pub and: AndGate,

    /// NOT gate to invert the lock signal for the AND gate.
    #[submodule]
    pub not: NotGate,
}
