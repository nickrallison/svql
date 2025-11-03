use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_macros::composite;

use crate::{
    Connection,
    Match,
    Search,
    State,
    Wire,
    WithPath,
    enum_composites::dff_any::DffAny,
    instance::Instance,
    security::primitives::locked_register::LockedRegister,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        enum_composite::SearchableEnumComposite,
        netlist::SearchableNetlist,
    }, // FIXED: traits::netlist
};

use crate::security::cwe1280::grant_access::GrantAccess;

pub mod grant_access;

// composite! {
//     name: Cwe1280,
//     subs: [
//         access_grant: GrantAccess,
//         locked_reg: LockedRegister,
//         reg_any: DffAny,
//     ],
//     connections: [
//         [
//             grant_access . grant => locked_reg . data_in,
//         ]
//     ]
// }

#[derive(Debug, Clone)]
pub struct Cwe1280<S>
where
    S: State,
{
    pub path: Instance,
    pub grant_access: GrantAccess<S>,
    pub locked_reg: LockedRegister<S>,
    pub reg_any: DffAny<S>,
}

impl<S> Cwe1280<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            grant_access: GrantAccess::new(path.child("grant_access".to_string())),
            locked_reg: LockedRegister::new(path.child("locked_reg".to_string())),
            reg_any: DffAny::new(path.child("reg_any".to_string())),
        }
    }
}

impl<S> WithPath<S> for Cwe1280<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
            Some("clk") => self.grant_access.find_port(p),
            Some("d") => self.locked_reg.find_port(p),
            Some("en") => self.reg_any.find_port(p),
            _ => None,
        }
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for Cwe1280<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        vec![vec![
            Connection {
                from: self.grant_access.grant.clone(),
                to: self.locked_reg.data_in().clone(),
            },
            Connection {
                from: self.locked_reg.data_out().clone(),
                to: self.reg_any.data_input().clone(),
            },
        ]]
    }
}

impl<'ctx> MatchedComposite<'ctx> for Cwe1280<Match<'ctx>> {}

impl SearchableComposite for Cwe1280<Search> {
    type Hit<'ctx> = Cwe1280<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        todo!();
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        todo!();
    }
}
