use svql_macros::netlist;

use crate::{State, Wire};

netlist! {
    name: GrantAccess,
    module_name: "grant_access",
    file: "examples/patterns/security/access_control/grant_access/verilog/grant_access.v",
    inputs: [usr_id, correct_id],
    outputs: [grant]
}
