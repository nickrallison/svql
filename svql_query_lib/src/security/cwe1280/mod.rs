/// Credentials validation logic for access control.
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
    /// Instance of the module that validates credentials.
    #[submodule]
    pub grant_access: GrantAccess,
    /// The intermediate delay register that keeps access stale.
    #[submodule]
    pub reg_any: DffAny,
    /// The register whose access control arrives too late.
    #[submodule]
    pub locked_reg: LockedRegister,
}
