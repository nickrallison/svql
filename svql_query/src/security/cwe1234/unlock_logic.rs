use crate::composites::rec_or::RecOr;

use crate::prelude::*;
use crate::traits::{MatchedComponent, SearchableComponent, kind};

use common::{Config, ModuleConfig};
use driver::{Context, Driver, DriverKey};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Represents the unlock/bypass logic pattern in CWE1234:
/// - Top-level AND gate (write enable)
/// - Recursive OR tree (bypass conditions)
/// - NOT gate somewhere in the OR tree (negated lock signal)
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UnlockLogic<S>
where
    S: State,
{
    pub path: Instance,
    pub top_and: AndGate<S>,
    pub rec_or: RecOr<S>,
    pub not_gate: NotGate<S>,
}

impl<S> Hardware for UnlockLogic<S>
where
    S: State,
{
    type State = S;

    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "UnlockLogic"
    }

    fn children(&self) -> Vec<&dyn Hardware<State = Self::State>> {
        vec![&self.top_and, &self.rec_or, &self.not_gate]
    }

    fn report(&self, name: &str) -> ReportNode {
        let children = vec![
            self.top_and.report("top_and"),
            self.not_gate.report("not_gate"),
            self.rec_or.report("rec_or"),
        ];

        ReportNode {
            name: name.to_string(),
            type_name: "UnlockLogic".to_string(),
            path: self.path.clone(),
            details: None,
            source_loc: self.top_and.y.source(),
            children,
        }
    }
}

impl SearchableComponent for UnlockLogic<Search> {
    type Kind = kind::Composite;
    type Match = UnlockLogic<Match>;

    fn create_at(base_path: Instance) -> Self {
        Self::new(base_path)
    }

    fn build_context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        let and_ctx = AndGate::<Search>::build_context(driver, config)?;
        let or_ctx = RecOr::<Search>::build_context(driver, config)?;
        let not_ctx = NotGate::<Search>::build_context(driver, config)?;

        Ok(and_ctx.merge(or_ctx).merge(not_ctx))
    }

    fn execute_search(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Match> {
        tracing::info!("UnlockLogic::execute: starting CWE1234 unlock pattern search");

        let haystack_index = context.get(key).unwrap().index();

        let and_gates = self.top_and.execute(driver, context, key, config);
        let rec_ors = self.rec_or.execute(driver, context, key, config);
        let not_gates = self.not_gate.execute(driver, context, key, config);

        tracing::info!(
            "UnlockLogic::execute: Found {} AND gates, {} RecOR trees, {} NOT gates",
            and_gates.len(),
            rec_ors.len(),
            not_gates.len()
        );

        let or_to_and_conn = Connection {
            from: self.rec_or.output().clone(),
            to: self.top_and.a.clone(),
        };

        #[cfg(feature = "parallel")]
        let or_iter = rec_ors.par_iter();
        #[cfg(not(feature = "parallel"))]
        let or_iter = rec_ors.iter();

        let rec_or_and_pairs: Vec<(RecOr<Match>, AndGate<Match>)> = {
            or_iter
                .enumerate()
                .flat_map(|(rec_or_index, rec_or)| {
                    if rec_or_index % 50 == 0 {
                        tracing::debug!(
                            "UnlockLogic::execute: Processing RecOr index {}",
                            rec_or_index
                        );
                    }

                    let from_wire = rec_or
                        .find_port(&or_to_and_conn.from.path)
                        .expect("RecOr output port not found");

                    let Some(from_info) = &from_wire.inner else {
                        return vec![];
                    };
                    let Some(from_wrapper) = haystack_index.get_cell_by_id(from_info.id) else {
                        return vec![];
                    };

                    let fanout = haystack_index
                        .fanout_set(&from_wrapper)
                        .expect("Fanout not found for RecOr cell");

                    let pairs: Vec<_> = and_gates
                        .iter()
                        .filter_map(|and_gate| {
                            let connected = [&and_gate.a, &and_gate.b].iter().any(|to_wire| {
                                let Some(to_info) = &to_wire.inner else {
                                    return false;
                                };
                                let Some(to_wrapper) = haystack_index.get_cell_by_id(to_info.id)
                                else {
                                    return false;
                                };
                                fanout.contains(&to_wrapper)
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
            "UnlockLogic::execute: Found {} valid (RecOr, AND) pairs",
            rec_or_and_pairs.len()
        );

        #[cfg(feature = "parallel")]
        let and_or_iter = rec_or_and_pairs.par_iter();
        #[cfg(not(feature = "parallel"))]
        let and_or_iter = rec_or_and_pairs.iter();

        let results: Vec<UnlockLogic<Match>> = {
            and_or_iter
                .enumerate()
                .flat_map(|(rec_or_and_index, (rec_or, top_and))| {
                    if rec_or_and_index % 50 == 0 {
                        tracing::debug!(
                            "UnlockLogic::execute: Processing pair index {}",
                            rec_or_and_index
                        );
                    }

                    let rec_or_fanin = rec_or.fanin_set(haystack_index);

                    let candidates: Vec<_> = not_gates
                        .iter()
                        .filter_map(|not_gate| {
                            let Some(not_info) = &not_gate.y.inner else {
                                return None;
                            };
                            let Some(not_wrapper) = haystack_index.get_cell_by_id(not_info.id)
                            else {
                                return None;
                            };

                            if !rec_or_fanin.contains(&not_wrapper) {
                                return None;
                            }

                            let candidate = UnlockLogic {
                                path: self.path.clone(),
                                top_and: top_and.clone(),
                                rec_or: rec_or.clone(),
                                not_gate: not_gate.clone(),
                            };

                            let mut builder = ConnectionBuilder {
                                constraints: Vec::new(),
                            };
                            candidate.define_connections(&mut builder);

                            let mut valid = true;
                            for group in builder.constraints {
                                let mut group_satisfied = false;
                                for (from, to) in group {
                                    if let (Some(f), Some(t)) = (from, to)
                                        && validate_connection(f, t, haystack_index)
                                    {
                                        group_satisfied = true;
                                        break;
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
            "UnlockLogic::execute: Found {} final valid patterns",
            results.len()
        );
        results
    }
}

impl MatchedComponent for UnlockLogic<Match> {
    type Search = UnlockLogic<Search>;
}

// --- Dehydrate/Rehydrate implementations ---

use crate::session::{
    Dehydrate, Rehydrate, DehydratedRow, MatchRow, QuerySchema, 
    WireFieldDesc, SubmoduleFieldDesc, RehydrateContext, SessionError
};

impl Dehydrate for UnlockLogic<Match> {
    const SCHEMA: QuerySchema = QuerySchema::new(
        "UnlockLogic",
        &[
            // Top AND gate wires
            WireFieldDesc { name: "top_and_a" },
            WireFieldDesc { name: "top_and_b" },
            WireFieldDesc { name: "top_and_y" },
            // NOT gate wires
            WireFieldDesc { name: "not_a" },
            WireFieldDesc { name: "not_y" },
        ],
        &[
            SubmoduleFieldDesc { name: "rec_or", type_name: "RecOr" },
        ],
    );
    
    fn dehydrate(&self) -> DehydratedRow {
        DehydratedRow::new(self.path.to_string())
            .with_wire("top_and_a", self.top_and.a.inner.as_ref().map(|c| c.id as u32))
            .with_wire("top_and_b", self.top_and.b.inner.as_ref().map(|c| c.id as u32))
            .with_wire("top_and_y", self.top_and.y.inner.as_ref().map(|c| c.id as u32))
            .with_wire("not_a", self.not_gate.a.inner.as_ref().map(|c| c.id as u32))
            .with_wire("not_y", self.not_gate.y.inner.as_ref().map(|c| c.id as u32))
            // rec_or submodule index must be set by caller
    }
}

impl Rehydrate for UnlockLogic<Match> {
    const TYPE_NAME: &'static str = "UnlockLogic";
    
    fn rehydrate(
        row: &MatchRow,
        ctx: &RehydrateContext<'_>,
    ) -> Result<Self, SessionError> {
        let path = Instance::from_path(&row.path);
        
        // Rehydrate top_and
        let top_and_path = path.child("top_and");
        let top_and = AndGate {
            path: top_and_path.clone(),
            a: ctx.rehydrate_wire(top_and_path.child("a"), row.wire("top_and_a")),
            b: ctx.rehydrate_wire(top_and_path.child("b"), row.wire("top_and_b")),
            y: ctx.rehydrate_wire(top_and_path.child("y"), row.wire("top_and_y")),
        };
        
        // Rehydrate not_gate
        let not_path = path.child("not_gate");
        let not_gate = NotGate {
            path: not_path.clone(),
            a: ctx.rehydrate_wire(not_path.child("a"), row.wire("not_a")),
            y: ctx.rehydrate_wire(not_path.child("y"), row.wire("not_y")),
        };
        
        // Rehydrate rec_or from submodule index
        let rec_or_idx = row.submodule("rec_or")
            .ok_or_else(|| SessionError::RehydrationError("Missing rec_or submodule index".into()))?;
        let rec_or = RecOr::rehydrate_by_index(rec_or_idx, ctx)?;
        
        Ok(UnlockLogic { path, top_and, rec_or, not_gate })
    }
}

impl<S> Topology<S> for UnlockLogic<S>
where
    S: State,
{
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        ctx.connect_any(&[
            (Some(self.rec_or.output()), Some(&self.top_and.a)),
            (Some(self.rec_or.output()), Some(&self.top_and.b)),
        ]);
    }
}

impl UnlockLogic<Search> {
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            top_and: AndGate::instantiate(path.child("top_and")),
            rec_or: RecOr::new(path.child("rec_or")),
            not_gate: NotGate::instantiate(path.child("not_gate")),
        }
    }
}

impl<'ctx> UnlockLogic<Match> {
    pub fn or_tree_depth(&self) -> usize {
        self.rec_or.depth()
    }
}
