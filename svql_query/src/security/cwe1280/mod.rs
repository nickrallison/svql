use svql_common::{Config, ModuleConfig};
use svql_driver::key::DriverKey;
use svql_driver::{Context, Driver};
use svql_macros::netlist;

use crate::{
    Connection, Match, Search, State, Wire, WithPath,
    enum_composites::dff_any::DffAny,
    instance::Instance,
    security::primitives::locked_register::LockedRegister,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        enum_composite::SearchableEnumComposite,
        netlist::SearchableNetlist,
    },
};

use itertools::iproduct;

netlist! {
    name: GrantAccess,
    module_name: "grant_access",
    file: "examples/patterns/security/access_control/grant_access/verilog/grant_access.v",
    inputs: [usr_id, correct_id],
    outputs: [grant]
}

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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::Search;
//     use svql_common::{Config, Dedupe, MatchLength};
//     use svql_driver::Driver;

//     fn init_test_logger() {
//         let _ = tracing_subscriber::fmt()
//             .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//             .with_test_writer()
//             .try_init();
//     }

//     // NEW: Basic structural test (assumes fixtures exist; adjust paths as needed)
//     #[test]
//     fn test_cwe1280_basic() {
//         init_test_logger();

//         let config = Config::builder()
//             .match_length(MatchLength::Exact)
//             .dedupe(Dedupe::All)
//             .build();

//         // Placeholder: Replace with actual fixture path/module
//         let fixture_path = "examples/fixtures/cwes/cwe1280/verilog/cwe1280_basic.v";
//         let module_name = "cwe1280_basic";

//         let driver = Driver::new_workspace().unwrap();
//         let (haystack_key, haystack_design) = driver
//             .get_or_load_design(fixture_path, module_name, &config.haystack_options)
//             .unwrap();

//         let context = Cwe1280::<Search>::context(&driver, &config.needle_options).unwrap();
//         let context = context.with_design(haystack_key.clone(), haystack_design);

//         let results = Cwe1280::<Search>::query(
//             &haystack_key,
//             &context,
//             Instance::root("cwe1280".to_string()),
//             &config,
//         );

//         // Assume 1 expected match for basic case
//         assert_eq!(results.len(), 1, "Should find 1 CWE-1280 bypass pattern");
//         let hit = &results[0];

//         println!("✓ CWE-1280 basic test passed: {} match(es)", results.len());
//     }
// }
