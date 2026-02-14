//! CWE-1234: Internal or Debug Modes Allow Override of Locks.

/// Logical structures facilitating the bypass of register locks.
pub mod unlock_logic;

use crate::security::primitives::locked_register::LockedRegister;
use svql_query::prelude::*;
use unlock_logic::UnlockLogic;

/// Top-level pattern for CWE-1234.
#[derive(Debug, Clone, Composite)]
#[connection(from = ["unlock_logic", "unlock"], to = ["locked_register", "write_en"])]
pub struct Cwe1234 {
    /// Hierarchical bypass or unlock logic.
    #[submodule]
    pub unlock_logic: UnlockLogic,
    /// The register whose lock is being bypassed.
    #[submodule]
    pub locked_register: LockedRegister,
}
