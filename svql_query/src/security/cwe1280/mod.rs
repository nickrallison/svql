use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_macros::composite;

use crate::{
    Connection,
    Match,
    Search,
    State,
    WithPath,
    enum_composites::dff_any::DffAny,
    instance::Instance,
    primitives::or::OrGate,
    security::primitives::locked_register::LockedRegister,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        netlist::SearchableNetlist,
    }, // FIXED: traits::netlist
};

use crate::security::cwe1280::grant_access::GrantAccess;

mod grant_access;

composite! {
    name: Cwe1280,
    subs: [
        access_grant: GrantAccess,
        locked_reg: LockedRegister,
        reg_any: DffAny,
    ],
    connections: [
        [
            sdffe . q => and_gate . a,
            sdffe . q => and_gate . b
        ]
    ]
}
