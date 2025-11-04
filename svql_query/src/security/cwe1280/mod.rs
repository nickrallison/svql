mod grant_access;

use svql_common::{Config, ModuleConfig};
use svql_driver::key::DriverKey;
use svql_driver::{Context, Driver};

use crate::security::cwe1280::grant_access::GrantAccess;
use crate::traits::netlist::SearchableNetlist;
use crate::{
    Connection, Match, Search, State, Wire, WithPath,
    enum_composites::dff_any::DffAny,
    instance::Instance,
    security::primitives::locked_register::LockedRegister,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        enum_composite::SearchableEnumComposite,
    },
};

use itertools::iproduct;

// Manual composite implementation for CWE-1280 (access control bypass via improper ID validation)
// - Detects: GrantAccess logic (usr_id == correct_id) feeding into locked register data_in
// - Chained to another DFF (reg_any) via data_out for potential escalation
// - Vulnerability: Weak ID comparison allows unauthorized data writes to protected register
// Note: Cannot use composite! macro directly due to enum subs (LockedRegister/DffAny) using accessor methods
//       (e.g., data_in()) instead of direct fields; manual impl required for connections/accessors

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
    // NEW: Constructor for search-time instantiation (dummy for validation)
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            grant_access: GrantAccess::new(path.child("grant_access".to_string())),
            locked_reg: LockedRegister::new(path.child("locked_reg".to_string())),
            reg_any: DffAny::new(path.child("reg_any".to_string())),
        }
    }

    // NEW: Helper to get the grant signal for reporting
    pub fn grant_signal(&self) -> &Wire<S> {
        &self.grant_access.grant
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
            Some("grant_access") => self.grant_access.find_port(p),
            Some("locked_reg") => self.locked_reg.find_port(p),
            Some("reg_any") => self.reg_any.find_port(p),
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
        // FIXED: Use accessors for enum subs (data_in(), data_out(), data_input())
        vec![vec![
            // Critical vuln connection: Grant signal -> locked reg data_in (bypass via ID match)
            Connection {
                from: self.grant_access.grant.clone(),
                to: self.locked_reg.data_in().clone(),
            },
            // Chain: Locked reg out -> secondary DFF in (escalation potential)
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
        // Merge contexts from all subs (GrantAccess is netlist; others are enum_composites)
        let access_ctx = GrantAccess::<Search>::context(driver, config)?;
        let locked_ctx = LockedRegister::<Search>::context(driver, config)?;
        let reg_ctx = DffAny::<Search>::context(driver, config)?;

        let mut iter = vec![access_ctx, locked_ctx, reg_ctx].into_iter();
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

        let grant_accesses = GrantAccess::<Search>::query(
            haystack_key,
            context,
            path.child("grant_access".to_string()),
            config,
        );

        let locked_regs = LockedRegister::<Search>::query(
            haystack_key,
            context,
            path.child("locked_reg".to_string()),
            config,
        );

        let reg_anys = DffAny::<Search>::query(
            haystack_key,
            context,
            path.child("reg_any".to_string()),
            config,
        );

        tracing::event!(
            tracing::Level::INFO,
            "Cwe1280::query: Found {} grant logics, {} locked regs, {} secondary DFFs",
            grant_accesses.len(),
            locked_regs.len(),
            reg_anys.len()
        );

        // Cartesian product (iproduct) of sub-queries, construct composite, validate connections
        iproduct!(grant_accesses, locked_regs, reg_anys)
            .map(|(ga, lr, ra)| Cwe1280 {
                path: path.clone(),
                grant_access: ga,
                locked_reg: lr,
                reg_any: ra,
            })
            .filter(|composite| {
                let valid = composite.validate_connections(composite.connections());
                if valid {
                    tracing::debug!(
                        "Cwe1280: Valid bypass pattern - grant({}) -> locked_reg({}) -> reg_any({})",
                        composite.grant_access.path.inst_path(),
                        composite.locked_reg.path().inst_path(),
                        composite.reg_any.path().inst_path()
                    );
                }
                valid
            })
            .collect()
    }
}
