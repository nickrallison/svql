use svql_query::prelude::*;

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/security/access_control/grant_access/rtlil/grant_access.il",
    module = "grant_access"
)]
pub struct GrantAccess {
    #[port(input)]
    pub usr_id: Wire,
    #[port(input)]
    pub correct_id: Wire,
    #[port(output)]
    pub grant: Wire,
}
