pub mod locked_register;
pub mod register;
pub mod unlock_logic;

use crate::{
    Connection, Match, Search, State, WithPath,
    composite::{Composite, MatchedComposite, SearchableComposite},
    instance::Instance,
};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

use locked_register::LockedRegister;
use unlock_logic::UnlockLogic;

/// Complete CWE-1234 pattern: Locked register with bypassable unlock logic
///
/// This composite detects the full vulnerability by combining:
/// 1. UnlockLogic: AND gate with OR tree containing negated lock signal
/// 2. LockedRegister: DFF that stores protected data
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
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            unlock_logic: UnlockLogic::new(path.child("unlock_logic".to_string())),
            locked_register: LockedRegister::new(path.child("locked_register".to_string())),
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
        // Critical connection: unlock logic output must feed the register's enable
        // This is what creates the vulnerability - bypass can override lock
        vec![vec![Connection {
            from: self.unlock_logic.top_and.y.clone(),
            to: self.locked_register.cell_wire().clone(),
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
        // Merge contexts from both sub-patterns
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

        // Query both sub-patterns independently
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

        // Combine patterns and validate connections
        let mut results = Vec::new();
        let mut candidates_checked = 0;
        let mut unlock_failures = 0;
        let mut connection_failures = 0;

        for unlock_logic in &unlock_patterns {
            // Pre-validate unlock logic has proper structure
            if !unlock_logic.has_not_in_or_tree() {
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
                // This filters out lock_status DFFs (whose enable is just Lock signal)
                // and keeps only data DFFs (whose enable comes from bypass logic)
                if !candidate.validate_connections(candidate.connections()) {
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
