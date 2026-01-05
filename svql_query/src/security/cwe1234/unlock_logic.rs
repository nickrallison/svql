use crate::composites::rec_or::RecOr;

use crate::prelude::*;

use std::sync::Arc;
use common::{Config, ModuleConfig};
use driver::{Context, Driver, DriverKey};

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
    pub top_and: AndGate<S>,
    pub rec_or: RecOr<S>,
    pub not_gate: NotGate<S>,
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
        ctx.connect_any(&[
            (Some(self.rec_or.output()), Some(&self.top_and.a)),
            (Some(self.rec_or.output()), Some(&self.top_and.b)),
        ]);
    }
}

impl Searchable for UnlockLogic<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self::new(base_path)
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

impl<'a> crate::traits::Reportable for UnlockLogic<Match> {
    fn to_report(&self, name: &str) -> crate::report::ReportNode {
        let children = vec![
            self.top_and.to_report("top_and"),
            self.not_gate.to_report("not_gate"),
            self.rec_or.to_report("rec_or"),
        ];

        crate::report::ReportNode {
            name: name.to_string(),
            type_name: "UnlockLogic".to_string(),
            path: self.path.clone(),
            details: None,
            source_loc: Some(
                self.top_and
                    .y
                    .inner
                    .as_ref()
                    .and_then(|c| c.get_source())
                    .unwrap_or_else(|| subgraph::cell::SourceLocation {
                        file: std::sync::Arc::from(""),
                        lines: Vec::new(),
                    }),
            ),
            children,
        }
    }
}

impl Query for UnlockLogic<Search> {
    type Matched<'a> = UnlockLogic<Match>;

    fn query<'a>(
        &self,
        driver: &Driver,
        context: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>> {
        tracing::info!("UnlockLogic::query: starting CWE1234 unlock pattern search");

        let haystack_index = context.get(key).unwrap().index();

        let and_gates = self.top_and.query(driver, context, key, config);
        let rec_ors = self.rec_or.query(driver, context, key, config);
        let not_gates = self.not_gate.query(driver, context, key, config);

        tracing::info!(
            "UnlockLogic::query: Found {} AND gates, {} RecOR trees, {} NOT gates",
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
                            "UnlockLogic::query: Processing RecOr index {}",
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
            "UnlockLogic::query: Found {} valid (RecOr, AND) pairs",
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
                            "UnlockLogic::query: Processing pair index {}",
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

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        let and_ctx = AndGate::<Search>::context(driver, config)?;
        let or_ctx = RecOr::<Search>::context(driver, config)?;
        let not_ctx = NotGate::<Search>::context(driver, config)?;

        Ok(and_ctx.merge(or_ctx).merge(not_ctx))
    }
}

impl<'ctx> UnlockLogic<Match> {
    pub fn or_tree_depth(&self) -> usize {
        self.rec_or.depth()
    }
}
