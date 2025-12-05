pub mod unlock_logic;

use crate::{
    State,
    instance::Instance,
    traits::{ConnectionBuilder, Topology},
};
use svql_macros::composite;

use crate::security::primitives::locked_register::LockedRegister;
use unlock_logic::UnlockLogic;

/// Complete CWE-1234 pattern: Locked register with bypassable unlock logic
///
/// This composite detects the full vulnerability by combining:
/// 1. UnlockLogic: AND gate with OR tree containing negated lock signal
/// 2. LockedRegister: DFF with enable signal that stores protected data
///
/// The vulnerability exists when the unlock logic output connects to the
/// register's enable input, allowing bypass conditions to override the lock.
#[composite]
pub struct Cwe1234<S: State> {
    #[path]
    pub path: Instance,

    #[submodule]
    pub unlock_logic: UnlockLogic<S>,

    #[submodule]
    pub locked_register: LockedRegister<S>,
}

impl<S: State> Topology<S> for Cwe1234<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        ctx.connect(
            Some(&self.unlock_logic.top_and.y),
            self.locked_register.write_en(),
        );
    }
}
