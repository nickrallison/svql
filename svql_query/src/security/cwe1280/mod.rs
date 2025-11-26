mod grant_access;

use crate::{
    State,
    instance::Instance,
    security::primitives::locked_register::LockedRegister,
    traits::{ConnectionBuilder, Topology},
    variants::dff_any::DffAny,
};
use svql_macros::composite;

use crate::security::cwe1280::grant_access::GrantAccess;

/// Represents the first stage of CWE-1280: Access granted and stored in a register.
#[composite]
pub struct DelayedGrantAccess<S: State> {
    #[path]
    pub path: Instance,

    #[submodule]
    pub grant_access: GrantAccess<S>,

    #[submodule]
    pub reg_any: DffAny<S>,
}

impl<S: State> Topology<S> for DelayedGrantAccess<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        // Grant logic output must feed intermediate DFF's data input
        // DffAny is a variant, so we use the accessor method .d()
        ctx.connect(Some(&self.grant_access.grant), self.reg_any.d());
    }
}

/// Complete CWE-1280 pattern: Access Control with Weakness (Stale Data)
///
/// Detects when a grant signal is stored in a register (DelayedGrantAccess)
/// and that stored (potentially stale) signal is used to enable a sensitive operation (LockedRegister).
#[composite]
pub struct Cwe1280<S: State> {
    #[path]
    pub path: Instance,

    #[submodule]
    pub delayed_grant_access: DelayedGrantAccess<S>,

    #[submodule]
    pub locked_reg: LockedRegister<S>,
}

impl<S: State> Topology<S> for Cwe1280<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        // The delayed grant signal (Q output of the intermediate register)
        // must connect to the write enable of the locked register.
        ctx.connect(
            self.delayed_grant_access.reg_any.q(),
            self.locked_reg.write_en(),
        );
    }
}
