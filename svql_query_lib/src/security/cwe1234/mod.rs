pub mod unlock_logic;

use svql_query::prelude::*;
use crate::security::primitives::locked_register::LockedRegister;
use unlock_logic::UnlockLogic;

#[derive(Debug, Clone, Composite)]
#[connection(from = ["unlock_logic", "unlock"], to = ["locked_register", "write_en"])]
pub struct Cwe1234 {
    #[submodule]
    pub unlock_logic: UnlockLogic,
    #[submodule]
    pub locked_register: LockedRegister,
}
