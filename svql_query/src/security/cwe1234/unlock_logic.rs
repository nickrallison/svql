// svql_query/src/queries/security/cwe1234/unlock_logic.rs

use crate::{
    composites::{Composite, MatchedComposite, SearchableComposite}, instance::Instance, netlist::SearchableNetlist, queries::netlist::basic::{and::AndGate, not::NotGate}, Connection,
    Match,
    Search,
    State,
    WithPath,
};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use crate::composites::rec_or::RecOr;

/// Represents the unlock/bypass logic pattern in CWE1234:
/// - Top-level AND gate (write enable)
/// - Recursive OR tree (bypass conditions)
/// - NOT gate somewhere in the OR tree (negated lock signal)
#[derive(Debug, Clone)]
pub struct UnlockLogic<S>
where
    S: State,
{
    pub path: Instance,
    pub top_and: AndGate<S>,  // Write enable gate
    pub rec_or: RecOr<S>,     // Recursive OR tree of bypass conditions
    pub not_gate: NotGate<S>, // Negated lock signal
}

impl<S> UnlockLogic<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            top_and: AndGate::new(path.child("top_and".to_string())),
            rec_or: RecOr::new(path.child("rec_or".to_string())),
            not_gate: NotGate::new(path.child("not_gate".to_string())),
        }
    }
}

impl<S> WithPath<S> for UnlockLogic<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
            Some("top_and") => self.top_and.find_port(p),
            Some("rec_or") => self.rec_or.find_port(p),
            Some("not_gate") => self.not_gate.find_port(p),
            _ => None,
        }
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for UnlockLogic<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        // The OR tree output must connect to one of the AND inputs
        vec![vec![
            Connection {
                from: self.rec_or.output().clone(),
                to: self.top_and.a.clone(),
            },
            Connection {
                from: self.rec_or.output().clone(),
                to: self.top_and.b.clone(),
            },
        ]]
        // Note: NOT gate to OR tree connection is validated separately
        // via has_not_in_or_tree() because it can connect at any depth
    }
}

impl<'ctx> MatchedComposite<'ctx> for UnlockLogic<Match<'ctx>> {}

impl<'ctx> UnlockLogic<Match<'ctx>> {
    /// Check if the NOT gate output connects to any level of the RecOr tree.
    /// This is the key validation for the CWE1234 pattern: there must be
    /// a negated lock signal somewhere in the bypass conditions.
    pub fn has_not_in_or_tree(&self) -> bool {
        tracing::debug!(
            "Checking if NOT gate (depth={}) connects to OR tree (depth={})",
            1,
            self.rec_or.depth()
        );
        self.check_not_connects_to_or(&self.rec_or, 1)
    }

    /// Recursively check if not_gate.y connects to this OR or any of its children.
    ///
    /// This traverses the entire RecOr tree, checking at each level if the
    /// NOT gate output connects to either input (a or b) of the OR gate.
    fn check_not_connects_to_or(&self, rec_or: &RecOr<Match<'ctx>>, depth: usize) -> bool {
        tracing::trace!(
            "Checking OR gate at depth {} (has_child={})",
            depth,
            rec_or.child.is_some()
        );

        // Check if not_gate.y connects to this level's OR inputs (a or b)
        let connects_to_a = self.validate_connection(Connection {
            from: self.not_gate.y.clone(),
            to: rec_or.or.a.clone(),
        });

        let connects_to_b = self.validate_connection(Connection {
            from: self.not_gate.y.clone(),
            to: rec_or.or.b.clone(),
        });

        if connects_to_a {
            tracing::debug!("NOT gate connects to OR input 'a' at depth {}", depth);
            return true;
        }

        if connects_to_b {
            tracing::debug!("NOT gate connects to OR input 'b' at depth {}", depth);
            return true;
        }

        // If not connected at this level, recursively check the child
        if let Some(ref child) = rec_or.child {
            tracing::trace!("Recursing into child at depth {}", depth + 1);
            return self.check_not_connects_to_or(child, depth + 1);
        }

        // No connection found at any level
        tracing::debug!(
            "NOT gate does not connect to OR tree (searched {} levels)",
            depth
        );
        false
    }

    /// Get the depth of the OR tree for debugging/reporting
    pub fn or_tree_depth(&self) -> usize {
        self.rec_or.depth()
    }
}

impl SearchableComposite for UnlockLogic<Search> {
    type Hit<'ctx> = UnlockLogic<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Need contexts for all three components
        let and_ctx = AndGate::<Search>::context(driver, config)?;
        let or_ctx = RecOr::<Search>::context(driver, config)?;
        let not_ctx = NotGate::<Search>::context(driver, config)?;

        Ok(and_ctx.merge(or_ctx).merge(not_ctx))
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::info!("UnlockLogic::query: starting CWE1234 unlock pattern search");

        // Query all three component types
        let and_gates = AndGate::<Search>::query(
            haystack_key,
            context,
            path.child("top_and".to_string()),
            config,
        );

        let rec_ors = RecOr::<Search>::query(
            haystack_key,
            context,
            path.child("rec_or".to_string()),
            config,
        );

        let not_gates = NotGate::<Search>::query(
            haystack_key,
            context,
            path.child("not_gate".to_string()),
            config,
        );

        tracing::info!(
            "UnlockLogic::query: Found {} AND gates, {} RecOR trees, {} NOT gates",
            and_gates.len(),
            rec_ors.len(),
            not_gates.len()
        );

        // Try all combinations (cartesian product)
        let mut results = Vec::new();
        let mut candidates_checked = 0;
        let mut connection_failures = 0;
        let mut not_in_tree_failures = 0;

        for top_and in &and_gates {
            for rec_or in &rec_ors {
                for not_gate in &not_gates {
                    candidates_checked += 1;

                    let candidate = UnlockLogic {
                        path: path.clone(),
                        top_and: top_and.clone(),
                        rec_or: rec_or.clone(),
                        not_gate: not_gate.clone(),
                    };

                    // First, validate the OR->AND connection
                    if !candidate.validate_connections(candidate.connections()) {
                        connection_failures += 1;
                        tracing::trace!(
                            "Candidate {}: OR->AND connection validation failed",
                            candidates_checked
                        );
                        continue;
                    }

                    // Second, validate the critical NOT->OR tree connection
                    if !candidate.has_not_in_or_tree() {
                        not_in_tree_failures += 1;
                        tracing::trace!(
                            "Candidate {}: NOT gate not found in OR tree (depth={})",
                            candidates_checked,
                            rec_or.depth()
                        );
                        continue;
                    }

                    tracing::debug!(
                        "Valid unlock pattern found: OR depth={}, AND={}, NOT present",
                        candidate.or_tree_depth(),
                        top_and.path.inst_path()
                    );

                    results.push(candidate);
                }
            }
        }

        tracing::info!(
            "UnlockLogic::query: Checked {} candidates, {} failed OR->AND, {} failed NOT-in-tree, {} valid",
            candidates_checked,
            connection_failures,
            not_in_tree_failures,
            results.len()
        );

        results
    }
}
