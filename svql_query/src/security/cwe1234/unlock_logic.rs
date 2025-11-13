// svql_query/src/queries/security/cwe1234/unlock_logic.rs

use crate::composites::rec_or::RecOr;
use crate::instance::Instance;
use crate::primitives::and::AndGate;
use crate::primitives::not::NotGate;
use crate::traits::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::traits::netlist::SearchableNetlist;
use crate::{Connection, Match, Search, State, WithPath};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::GraphIndex;
use svql_subgraph::cell::CellWrapper;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

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
    pub fn has_not_in_or_tree(&self, haystack_index: &GraphIndex<'ctx>) -> bool {
        tracing::debug!(
            "Checking if NOT gate (depth={}) connects to OR tree (depth={})",
            1,
            self.rec_or.depth()
        );
        self.check_not_connects_to_or(&self.rec_or, 1, haystack_index)
    }

    /// Recursively check if not_gate.y connects to this OR or any of its children.
    ///
    /// This traverses the entire RecOr tree, checking at each level if the
    /// NOT gate output connects to either input (a or b) of the OR gate.
    fn check_not_connects_to_or(
        &self,
        rec_or: &RecOr<Match<'ctx>>,
        depth: usize,
        haystack_index: &GraphIndex<'ctx>,
    ) -> bool {
        tracing::trace!(
            "Checking OR gate at depth {} (has_child={})",
            depth,
            rec_or.child.is_some()
        );

        // Check if not_gate.y connects to this level's OR inputs (a or b)
        let connects_to_a = self.validate_connection(
            Connection {
                from: self.not_gate.y.clone(),
                to: rec_or.or.a.clone(),
            },
            haystack_index,
        );

        let connects_to_b = self.validate_connection(
            Connection {
                from: self.not_gate.y.clone(),
                to: rec_or.or.b.clone(),
            },
            haystack_index,
        );

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
            return self.check_not_connects_to_or(child, depth + 1, haystack_index);
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

        let haystack_index = context.get(haystack_key).unwrap().index();

        // Query all components
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

        // Step 1: Filter RecOr -> AND connections
        let temp_self: Self = Self::new(path.clone());
        let or_to_and_conn = Connection {
            from: temp_self.rec_or.output().clone(),
            to: temp_self.top_and.a.clone(),
        };

        #[cfg(feature = "parallel")]
        let or_iter = rec_ors.par_iter();
        #[cfg(not(feature = "parallel"))]
        let or_iter = rec_ors.iter();

        let rec_or_and_pairs: Vec<(RecOr<Match<'ctx>>, AndGate<Match<'ctx>>)> = {
            or_iter
                .enumerate()
                .flat_map(|(rec_or_index, rec_or)| {
                    // Get RecOr output cell
                    let from_wire = rec_or
                        .find_port(&or_to_and_conn.from.path)
                        .expect("RecOr output port not found");
                    let from_cell: &CellWrapper<'ctx> = from_wire
                        .val
                        .as_ref()
                        .expect("RecOr output cell not found")
                        .design_node_ref
                        .as_ref()
                        .expect("RecOr design node not found");

                    let fanout = haystack_index
                        .fanout_set(from_cell)
                        .expect("Fanout not found for RecOr cell");

                    let pairs: Vec<_> = and_gates
                        .iter()
                        .filter_map(|and_gate| {
                            // Check both AND inputs (a and b) for commutative matching
                            let connected =
                                [&and_gate.a.path, &and_gate.b.path]
                                    .iter()
                                    .any(|and_input_path| {
                                        let to_wire = and_gate
                                            .find_port(and_input_path)
                                            .expect("AND input port not found");
                                        let to_cell: &CellWrapper<'ctx> = to_wire
                                            .val
                                            .as_ref()
                                            .expect("AND input cell not found")
                                            .design_node_ref
                                            .as_ref()
                                            .expect("AND design node not found");

                                        fanout.contains(to_cell)
                                    });

                            if connected {
                                Some((rec_or.clone(), and_gate.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if rec_or_index % 50 == 0 {
                        tracing::debug!(
                            "UnlockLogic::query: Processing RecOr index {} (parallel)...",
                            rec_or_index
                        );
                    }

                    pairs
                })
                .collect()
        };

        tracing::info!(
            "UnlockLogic::query: Found {} valid (RecOr, AND) pairs after connection filtering",
            rec_or_and_pairs.len()
        );

        #[cfg(feature = "parallel")]
        let and_or_iter = rec_or_and_pairs.par_iter();
        #[cfg(not(feature = "parallel"))]
        let and_or_iter = rec_or_and_pairs.iter();

        let results: Vec<UnlockLogic<Match<'ctx>>> = {
            and_or_iter
                .enumerate()
                .flat_map(|(rec_or_and_index, (rec_or, top_and))| {
                    let rec_or_fanin = rec_or.fanin_set(haystack_index);

                    let candidates: Vec<_> = not_gates
                        .iter()
                        .filter_map(|not_gate| {
                            // Check if NOT gate's output is in RecOr's fan-in set
                            let not_output_cell = not_gate
                                .y
                                .val
                                .as_ref()
                                .expect("NOT output not found")
                                .design_node_ref
                                .as_ref()
                                .expect("Design node not found");

                            if !rec_or_fanin.contains(not_output_cell) {
                                return None;
                            }

                            let candidate = UnlockLogic {
                                path: path.clone(),
                                top_and: top_and.clone(),
                                rec_or: rec_or.clone(),
                                not_gate: not_gate.clone(),
                            };

                            if candidate
                                .validate_connections(candidate.connections(), haystack_index)
                            {
                                tracing::trace!(
                                    "Valid unlock pattern found: OR depth={}, AND={}",
                                    candidate.or_tree_depth(),
                                    top_and.path.inst_path()
                                );
                                Some(candidate)
                            } else {
                                None
                            }
                        })
                        .collect();

                    if rec_or_and_index % 50 == 0 {
                        tracing::debug!(
                            "UnlockLogic::query: Processing pair index {} (parallel)...",
                            rec_or_and_index
                        );
                    }

                    candidates
                })
                .collect()
        };

        tracing::info!(
            "UnlockLogic::query: Found {} final valid patterns",
            results.len()
        );

        results
    }
}
