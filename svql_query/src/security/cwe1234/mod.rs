pub mod unlock_logic;

use crate::{
    Connection, Match, Search, State, WithPath,
    instance::Instance,
    traits::{
        composite::{Composite, MatchedComposite, SearchableComposite},
        enum_composite::SearchableEnumComposite,
    },
};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

use crate::security::primitives::locked_register::LockedRegister;
use unlock_logic::UnlockLogic;

/// Complete CWE-1234 pattern: Locked register with bypassable unlock logic
///
/// This composites detects the full vulnerability by combining:
/// 1. UnlockLogic: AND gate with OR tree containing negated lock signal
/// 2. LockedRegister: DFF with enable signal that stores protected data
///
/// The vulnerability exists when the unlock logic output connects to the
/// register's enable input, allowing bypass conditions to override the lock.
#[derive(Debug, Clone)]
pub struct Cwe1234<S>
where
    S: State,
{
    pub path: Instance,
    pub unlock_logic: UnlockLogic<S>,
    pub locked_register: LockedRegister<S>,
}

impl<S> Cwe1234<S>
where
    S: State,
{
    pub fn new(path: Instance, reg_any: LockedRegister<S>) -> Self {
        Self {
            path: path.clone(),
            unlock_logic: UnlockLogic::new(path.child("unlock_logic".to_string())),
            locked_register: reg_any,
        }
    }
}

impl<S> WithPath<S> for Cwe1234<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
            Some("unlock_logic") => self.unlock_logic.find_port(p),
            Some("locked_register") => self.locked_register.find_port(p),
            _ => None,
        }
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for Cwe1234<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        // Critical connection: unlock logic output → register enable
        // This is the vulnerability - bypass logic controls when data can be written
        vec![vec![Connection {
            from: self.unlock_logic.top_and.y.clone(),
            to: self.locked_register.enable_wire().clone(), // FIXED: Use enable_wire()
        }]]
    }
}

impl<'ctx> MatchedComposite<'ctx> for Cwe1234<Match<'ctx>> {}

impl SearchableComposite for Cwe1234<Search> {
    type Hit<'ctx> = Cwe1234<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        let unlock_ctx = UnlockLogic::<Search>::context(driver, config)?;
        let register_ctx = LockedRegister::<Search>::context(driver, config)?;

        Ok(unlock_ctx.merge(register_ctx))
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::info!("Cwe1234::query: starting complete CWE-1234 vulnerability search");

        let haystack_index = context.get(haystack_key).unwrap().index();

        let unlock_patterns = UnlockLogic::<Search>::query(
            haystack_key,
            context,
            path.child("unlock_logic".to_string()),
            config,
        );

        let registers = LockedRegister::<Search>::query(
            haystack_key,
            context,
            path.child("locked_register".to_string()),
            config,
        );

        tracing::info!(
            "Cwe1234::query: Found {} unlock patterns, {} registers",
            unlock_patterns.len(),
            registers.len()
        );

        let mut results = Vec::new();
        let mut candidates_checked = 0;
        let mut unlock_failures = 0;
        let mut connection_failures = 0;

        for unlock_logic in &unlock_patterns {
            if !unlock_logic.has_not_in_or_tree(haystack_index) {
                unlock_failures += 1;
                continue;
            }

            for locked_register in &registers {
                candidates_checked += 1;

                let candidate = Cwe1234 {
                    path: path.clone(),
                    unlock_logic: unlock_logic.clone(),
                    locked_register: locked_register.clone(),
                };

                // Validate the critical connection: unlock output → register enable
                if !candidate.validate_connections(candidate.connections(), haystack_index) {
                    connection_failures += 1;
                    tracing::trace!(
                        "Candidate {}: unlock→register connection failed",
                        candidates_checked
                    );
                    continue;
                }

                tracing::debug!(
                    "Valid CWE-1234 vulnerability: OR depth={}, register={}",
                    candidate.unlock_logic.or_tree_depth(),
                    candidate.locked_register.register_type(),
                );

                results.push(candidate);
            }
        }

        tracing::info!(
            "Cwe1234::query: Checked {} candidates, {} unlock failures, {} connection failures, {} vulnerabilities found",
            candidates_checked,
            unlock_failures,
            connection_failures,
            results.len()
        );

        results
    }
}
