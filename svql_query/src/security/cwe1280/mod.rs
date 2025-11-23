mod grant_access;

use svql_common::{Config, ModuleConfig};
use svql_driver::key::DriverKey;
use svql_driver::{Context, Driver};

use crate::security::cwe1280::grant_access::GrantAccess;
use crate::traits::composite::filter_out_by_connection;
use crate::traits::netlist::SearchableNetlist;
use crate::{
    Connection, Match, Search, State, Wire, WithPath,
    variants::dff_any::DffAny,
    instance::Instance,
    security::primitives::locked_register::LockedRegister,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        variant::SearchableVariant,
    },
};

#[derive(Debug, Clone)]
pub struct DelayedGrantAccess<S>
where
    S: State,
{
    pub path: Instance,
    pub grant_access: GrantAccess<S>,
    pub reg_any: DffAny<S>,
}

impl<S> DelayedGrantAccess<S>
where
    S: State,
{
    // NEW: Constructor for search-time instantiation (dummy for validation)
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            grant_access: GrantAccess::new(path.child("delayed_grant_access".to_string())),
            reg_any: DffAny::new(path.child("reg_any".to_string())),
        }
    }

    pub fn delayed_grant_signal(&self) -> &Wire<S> {
        &self.reg_any.output()
    }
}

impl<'ctx, S> DelayedGrantAccess<S>
where
    S: State,
{
    pub fn connection(grant_access: &GrantAccess<S>, reg_any: &DffAny<S>) -> Connection<S> {
        Connection {
            from: grant_access.grant.clone(),
            to: reg_any.data_input().clone(),
        }
    }
}

impl<S> WithPath<S> for DelayedGrantAccess<S>
where
    S: State,
{
    // FIXED: Correct match arms for sub-names (was incorrect "clk"/"d"/"en")
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
            Some("grant_access") => self.grant_access.find_port(p),
            Some("reg_any") => self.reg_any.find_port(p),
            _ => None,
        }
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for DelayedGrantAccess<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        // FIXED: Use accessors for enum subs (data_in(), data_out(), data_input())
        vec![
            // Grant logic output must feed intermediate DFF's data input
            vec![Connection {
                from: self.grant_access.grant.clone(),
                to: self.reg_any.data_input().clone(), // Grant stored in DFF
            }],
        ]
    }
}

impl<'ctx> MatchedComposite<'ctx> for DelayedGrantAccess<Match<'ctx>> {}

impl SearchableComposite for DelayedGrantAccess<Search> {
    type Hit<'ctx> = DelayedGrantAccess<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Merge contexts from all subs (GrantAccess is netlist; others are variants)
        let access_ctx = GrantAccess::<Search>::context(driver, config)?;
        let reg_ctx = DffAny::<Search>::context(driver, config)?;

        let mut iter = vec![access_ctx, reg_ctx].into_iter();
        let mut result = iter.next().ok_or("No sub-patterns defined")?;
        for ctx in iter {
            result = result.merge(ctx);
        }
        Ok(result)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        let haystack_index = context.get(haystack_key).unwrap().index();

        let grant_accesses = GrantAccess::<Search>::query(
            haystack_key,
            context,
            path.child("grant_access".to_string()),
            config,
        );

        let reg_anys = DffAny::<Search>::query(
            haystack_key,
            context,
            path.child("reg_any".to_string()),
            config,
        );

        let temp_self: Self = Self::new(path.clone());
        let conn = DelayedGrantAccess::connection(&temp_self.grant_access, &temp_self.reg_any);

        let merged_grant_accesses: Vec<(GrantAccess<Match<'ctx>>, DffAny<Match<'ctx>>)> =
            filter_out_by_connection::<GrantAccess<Match<'ctx>>, DffAny<Match<'ctx>>>(
                haystack_index,
                conn,
                grant_accesses,
                reg_anys,
            );

        // Cartesian product (iproduct) of sub-queries, construct composite, validate connections
        merged_grant_accesses
            .into_iter()
            .map(|(ga, ra)| DelayedGrantAccess {
                path: path.clone(),
                grant_access: ga,
                reg_any: ra,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Cwe1280<S>
where
    S: State,
{
    pub path: Instance,
    pub delayed_grant_access: DelayedGrantAccess<S>,
    pub locked_reg: LockedRegister<S>,
}

impl<S> Cwe1280<S>
where
    S: State,
{
    // NEW: Constructor for search-time instantiation (dummy for validation)
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            delayed_grant_access: DelayedGrantAccess::new(
                path.child("delayed_grant_access".to_string()),
            ),
            locked_reg: LockedRegister::new(path.child("locked_reg".to_string())),
        }
    }

    pub fn connection(
        delayed_grant_access: &DelayedGrantAccess<S>,
        locked_reg: &LockedRegister<S>,
    ) -> Connection<S> {
        Connection {
            from: delayed_grant_access.delayed_grant_signal().clone(),
            to: locked_reg.enable_wire().clone(),
        }
    }
}

impl<S> WithPath<S> for Cwe1280<S>
where
    S: State,
{
    // FIXED: Correct match arms for sub-names (was incorrect "clk"/"d"/"en")
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
            Some("delayed_grant_access") => self.delayed_grant_access.find_port(p),
            Some("locked_reg") => self.locked_reg.find_port(p),
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
        vec![
            // That DFF's output must feed locked register's enable/control
            vec![Connection {
                from: self.delayed_grant_access.delayed_grant_signal().clone(),
                to: self.locked_reg.enable_wire().clone(), // Stale grant controls access
            }],
        ]
    }
}

impl<'ctx> MatchedComposite<'ctx> for Cwe1280<Match<'ctx>> {}

impl SearchableComposite for Cwe1280<Search> {
    type Hit<'ctx> = Cwe1280<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Merge contexts from all subs (GrantAccess is netlist; others are variants)
        let delayed_grant_access_ctx = DelayedGrantAccess::<Search>::context(driver, config)?;
        let locked_ctx = LockedRegister::<Search>::context(driver, config)?;

        let mut iter = vec![delayed_grant_access_ctx, locked_ctx].into_iter();
        let mut result = iter.next().ok_or("No sub-patterns defined")?;
        for ctx in iter {
            result = result.merge(ctx);
        }
        Ok(result)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        // NEW: Sequential queries (similar to macro-generated non-parallel path)
        tracing::event!(
            tracing::Level::INFO,
            "Cwe1280::query: executing sequential queries for access bypass pattern"
        );

        let haystack_index = context.get(haystack_key).unwrap().index();

        let delayed_grant_accesses = DelayedGrantAccess::<Search>::query(
            haystack_key,
            context,
            path.child("delayed_grant_access".to_string()),
            config,
        );

        let locked_regs = LockedRegister::<Search>::query(
            haystack_key,
            context,
            path.child("locked_reg".to_string()),
            config,
        );

        let temp_self: Self = Self::new(path.clone());
        let conn = Cwe1280::connection(&temp_self.delayed_grant_access, &temp_self.locked_reg);

        let merged_grant_accesses: Vec<(
            DelayedGrantAccess<Match<'ctx>>,
            LockedRegister<Match<'ctx>>,
        )> = filter_out_by_connection::<DelayedGrantAccess<Match<'ctx>>, LockedRegister<Match<'ctx>>>(
            haystack_index,
            conn,
            delayed_grant_accesses,
            locked_regs,
        );

        // Cartesian product (iproduct) of sub-queries, construct composite, validate connections
        merged_grant_accesses
            .into_iter()
            .map(|(ga, ra)| Cwe1280 {
                path: path.clone(),
                delayed_grant_access: ga,
                locked_reg: ra,
            })
            .collect()
    }
}
