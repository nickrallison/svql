//! Credential validation logic for access control modules.

use svql_query::prelude::*;

/// Pattern for a module that validates credentials and outputs a grant signal.
#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/security/access_control/grant_access/rtlil/grant_access.il",
    module = "grant_access"
)]
pub struct GrantAccess {
    /// Input user/attacker ID.
    #[port(input)]
    pub usr_id: Wire,
    /// Hardcoded or register-based correct ID.
    #[port(input)]
    pub correct_id: Wire,
    /// High if IDs match.
    #[port(output)]
    pub grant: Wire,
}
