pub mod grant_access;

use crate::{
    State,
    instance::Instance,
    primitives::dff::DffAny,
    security::primitives::locked_register::LockedRegister,
    traits::{ConnectionBuilder, Topology},
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
        ctx.connect(Some(&self.grant_access.grant), self.reg_any.d());
    }
}

/// Complete CWE-1280 pattern: Access Control with Stale Access Check
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
        ctx.connect(
            self.delayed_grant_access.reg_any.q(),
            self.locked_reg.write_en(),
        );
    }
}
