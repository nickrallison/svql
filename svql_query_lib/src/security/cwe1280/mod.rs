pub mod grant_access;

use crate::primitives::dff::DffAny;
use crate::security::cwe1280::grant_access::GrantAccess;
use crate::security::primitives::locked_register::LockedRegister;
use svql_query::prelude::*;

/// Complete CWE-1280 pattern: Access Control with Stale Access Check
#[derive(Debug, Clone, Composite)]
#[connection(from = ["reg_any", "q"], to = ["locked_reg", "write_en"])]
#[connection(from = ["grant_access", "grant"], to = ["reg_any", "d"])]
pub struct Cwe1280 {
    #[submodule]
    pub grant_access: GrantAccess,
    #[submodule]
    pub reg_any: DffAny,
    #[submodule]
    pub locked_reg: LockedRegister,
}
