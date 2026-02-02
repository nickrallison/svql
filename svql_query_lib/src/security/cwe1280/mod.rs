pub mod grant_access;

use svql_query::prelude::*;
use crate::primitives::dff::DffAny;
use crate::security::cwe1280::grant_access::GrantAccess;
use crate::security::primitives::locked_register::LockedRegister;

/// Represents the first stage of CWE-1280: Access granted and stored in a register.
#[derive(Debug, Clone, Composite)]
#[connection(from = ["grant_access", "grant"], to = ["reg_any", "d"])]
pub struct DelayedGrantAccess {
    #[submodule]
    pub grant_access: GrantAccess,
    #[submodule]
    pub reg_any: DffAny,
    #[alias(output, target = ["reg_any", "q"])]
    pub grant_reg: Wire,
}

/// Complete CWE-1280 pattern: Access Control with Stale Access Check
#[derive(Debug, Clone, Composite)]
#[connection(from = ["delayed_grant_access", "grant_reg"], to = ["locked_reg", "write_en"])]
pub struct Cwe1280 {
    #[submodule]
    pub delayed_grant_access: DelayedGrantAccess,
    #[submodule]
    pub locked_reg: LockedRegister,
}
