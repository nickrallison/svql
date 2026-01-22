use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(
    file = "examples/patterns/security/access_control/grant_access/rtlil/grant_access.il",
    name = "grant_access"
)]
pub struct GrantAccess<S: State> {
    pub usr_id: Wire<S>,
    pub correct_id: Wire<S>,
    pub grant: Wire<S>,
}
