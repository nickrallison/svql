use crate::composites::rec_or::RecOr;
use crate::instance::Instance;
use crate::primitives::and::AndGate;
use crate::primitives::not::NotGate;
use crate::traits::{
    Component, ConnectionBuilder, Query, Searchable, Topology, validate_connection,
};
use crate::{Connection, Match, Search, State, Wire};
use std::sync::Arc;
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::GraphIndex;

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

impl<S> Component<S> for UnlockLogic<S>
where
    S: State,
{
    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "UnlockLogic"
    }

    fn children(&self) -> Vec<&dyn Component<S>> {
        vec![&self.top_and, &self.rec_or, &self.not_gate]
    }

    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
            Some("top_and") => self.top_and.find_port(p),
            Some("rec_or") => self.rec_or.find_port(p),
            Some("not_gate") => self.not_gate.find_port(p),
            _ => None,
        }
    }

    fn find_port_inner(&self, _rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        None
    }
}

impl<S> Topology<S> for UnlockLogic<S>
where
    S: State,
{
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        // The OR tree output must connect to one of the AND inputs
        ctx.connect_any(&[
            (Some(self.rec_or.output()), Some(&self.top_and.a)),
            (Some(self.rec_or.output()), Some(&self.top_and.b)),
        ]);
    }
}

impl Searchable for UnlockLogic<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self {
            path: base_path.clone(),
            top_and: AndGate::new(base_path.child("top_and")),
            rec_or: RecOr::new(base_path.child("rec_or")),
            not_gate: NotGate::new(base_path.child("not_gate")),
        }
    }
}

impl UnlockLogic<Search> {
    pub fn new(path: Instance) -> Self {
        <Self as Searchable>::instantiate(path)
    }

    pub fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Need contexts for all three components
        let and_ctx = AndGate::<Search>::context(driver, config)?;
        let or_ctx = RecOr::<Search>::context(driver, config)?;
        let not_ctx = NotGate::<Search>::context(driver, config)?;

        Ok(and_ctx.merge(or_ctx).merge(not_ctx))
    }
}

impl Query for UnlockLogic<Search> {
    type Matched<'a> = UnlockLogic<Match<'a>>;

    fn query<'a>(
        &self,
        driver: &Driver,
        context: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>> {
        tracing::info!("UnlockLogic::query: starting CWE1234 unlock pattern search");

        let haystack_index = context.get(key).unwrap().index();

        // Query all components
        let and_gates = self.top_and.query(driver, context, key, config);
        let rec_ors = self.rec_or.query(driver, context, key, config);
        let not_gates = self.not_gate.query(driver, context, key, config);

        tracing::info!(
            "UnlockLogic::query: Found {} AND gates, {} RecOR trees, {} NOT gates",
            and_gates.len(),
            rec_ors.len(),
            not_gates.len()
        );

        // Step 1: Filter RecOr -> AND connections
        // We use the Search instance (self) to define the connection pattern
        let or_to_and_conn = Connection {
            from: self.rec_or.output().clone(),
            to: self.top_and.a.clone(), // Just a placeholder for the path
        };

        #[cfg(feature = "parallel")]
        let or_iter = rec_ors.par_iter();
        #[cfg(not(feature = "parallel"))]
        let or_iter = rec_ors.iter();

        let rec_or_and_pairs: Vec<(RecOr<Match<'a>>, AndGate<Match<'a>>)> = {
            or_iter
                .enumerate()
                .flat_map(|(rec_or_index, rec_or)| {
                    if rec_or_index % 50 == 0 {
                        tracing::debug!(
                            "UnlockLogic::query: Processing RecOr index {}",
                            rec_or_index
                        );
                    }

                    let from_wire = rec_or
                        .find_port(&or_to_and_conn.from.path)
                        .expect("RecOr output port not found");
                    let from_cell = &from_wire.inner;
                    let fanout = haystack_index
                        .fanout_set(from_cell)
                        .expect("Fanout not found for RecOr cell");

                    let pairs: Vec<_> = and_gates
                        .iter()
                        .filter_map(|and_gate| {
                            // Check both AND inputs (a and b)
                            let connected = [&and_gate.a, &and_gate.b].iter().any(|to_wire| {
                                let to_cell = &to_wire.inner;
                                fanout.contains(to_cell)
                            });

                            if connected {
                                Some((rec_or.clone(), and_gate.clone()))
                            } else {
                                None
                            }
                        })
                        .collect();

                    pairs
                })
                .collect()
        };

        tracing::info!(
            "UnlockLogic::query: Found {} valid (RecOr, AND) pairs",
            rec_or_and_pairs.len()
        );

        #[cfg(feature = "parallel")]
        let and_or_iter = rec_or_and_pairs.par_iter();
        #[cfg(not(feature = "parallel"))]
        let and_or_iter = rec_or_and_pairs.iter();

        let results: Vec<UnlockLogic<Match<'a>>> = {
            and_or_iter
                .enumerate()
                .flat_map(|(rec_or_and_index, (rec_or, top_and))| {
                    if rec_or_and_index % 50 == 0 {
                        tracing::debug!(
                            "UnlockLogic::query: Processing pair index {}",
                            rec_or_and_index
                        );
                    }

                    let rec_or_fanin = rec_or.fanin_set(haystack_index);

                    let candidates: Vec<_> = not_gates
                        .iter()
                        .filter_map(|not_gate| {
                            let not_output_cell = &not_gate.y.inner;

                            if !rec_or_fanin.contains(not_output_cell) {
                                return None;
                            }

                            let candidate = UnlockLogic {
                                path: self.path.clone(),
                                top_and: top_and.clone(),
                                rec_or: rec_or.clone(),
                                not_gate: not_gate.clone(),
                            };

                            // Validate connections using Topology trait
                            let mut builder = ConnectionBuilder {
                                constraints: Vec::new(),
                            };
                            candidate.define_connections(&mut builder);

                            let mut valid = true;
                            for group in builder.constraints {
                                let mut group_satisfied = false;
                                for (from, to) in group {
                                    if let (Some(f), Some(t)) = (from, to) {
                                        if validate_connection(f, t, haystack_index) {
                                            group_satisfied = true;
                                            break;
                                        }
                                    }
                                }
                                if !group_satisfied {
                                    valid = false;
                                    break;
                                }
                            }

                            if valid { Some(candidate) } else { None }
                        })
                        .collect();
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

impl<'ctx> UnlockLogic<Match<'ctx>> {
    pub fn or_tree_depth(&self) -> usize {
        self.rec_or.depth()
    }
}
