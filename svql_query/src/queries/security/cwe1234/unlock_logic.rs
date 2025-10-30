use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

use crate::{
    Connection, Match, Search, State, WithPath,
    composite::{Composite, MatchedComposite, SearchableComposite},
    instance::Instance,
    netlist::SearchableNetlist,
    queries::{
        composite::rec_or::RecOr, // Reuse existing RecOr
        netlist::basic::{and::AndGate, not::NotGate, or::OrGate}, // Add OrGate if not imported
    },
};
use itertools::iproduct;
use tracing::{debug, info, info_span, warn};

// Composite for CWE-1234 unlock logic: write & (~lock_status | scan_mode | debug_unlocked)
// Updated: Flexible depth (1 or 2 for 3-input OR), explicit connection logging, fallback for flat OR.
#[derive(Debug, Clone)]
pub struct UnlockLogic<S>
where
    S: State,
{
    pub path: Instance,
    pub and: AndGate<S>,
    pub or_tree: RecOr<S>, // Recursive OR tree (filter for depth=1 or 2)
    pub not: NotGate<S>,
    // Open input wires
    pub write: crate::Wire<S>,
    pub lock_status: crate::Wire<S>,
    pub scan_mode: crate::Wire<S>,
    pub debug_unlocked: crate::Wire<S>,
}

impl<S> UnlockLogic<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        let and_path = path.child("and".to_string());
        let or_tree_path = path.child("or_tree".to_string());
        let not_path = path.child("not".to_string());

        Self {
            path: path.clone(),
            and: AndGate::new(and_path),
            or_tree: RecOr::new(or_tree_path),
            not: NotGate::new(not_path),
            write: crate::Wire::new(path.child("write".to_string())),
            lock_status: crate::Wire::new(path.child("lock_status".to_string())),
            scan_mode: crate::Wire::new(path.child("scan_mode".to_string())),
            debug_unlocked: crate::Wire::new(path.child("debug_unlocked".to_string())),
        }
    }

    // Get the enable output (AND.y)
    pub fn enable(&self) -> &crate::Wire<S> {
        &self.and.y
    }

    // Get the OR tree output
    pub fn or_output(&self) -> &crate::Wire<S> {
        self.or_tree.output()
    }

    // Get the base (leaf) OR gate: Traverse to innermost OR
    pub fn base_or(&self) -> &OrGate<S> {
        let mut current = &self.or_tree;
        while let Some(child) = &current.child {
            current = child;
        }
        &current.or // Assumes OrGate is public in RecOr
    }

    // Updated: Flexible depth check for 3-input OR (flat or tree)
    pub fn is_valid_depth(&self) -> bool {
        let depth = self.or_tree.depth();
        depth == 1 || depth == 2 // Accept flat (1 OR) or tree (2 ORs)
    }
}

impl<S> WithPath<S> for UnlockLogic<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
            Some("and") => self.and.find_port(p),
            Some("or_tree") => self.or_tree.find_port(p),
            Some("not") => self.not.find_port(p),
            Some("write") => Some(&self.write),
            Some("lock_status") => Some(&self.lock_status),
            Some("scan_mode") => Some(&self.scan_mode),
            Some("debug_unlocked") => Some(&self.debug_unlocked),
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
        vec![
            // AND connections
            vec![
                Connection {
                    from: self.write.clone(),
                    to: self.and.a.clone(),
                },
                Connection {
                    from: self.or_output().clone(),
                    to: self.and.b.clone(),
                },
            ],
            // NOT connection
            vec![Connection {
                from: self.lock_status.clone(),
                to: self.not.a.clone(),
            }],
            // Base OR connections (negated first input)
            vec![
                Connection {
                    from: self.not.y.clone(),
                    to: self.base_or().a.clone(),
                },
                Connection {
                    from: self.scan_mode.clone(),
                    to: self.base_or().b.clone(),
                },
            ],
            // Top OR direct input (debug_unlocked to .b; .a from child or base)
            vec![Connection {
                from: self.debug_unlocked.clone(),
                to: self.or_tree.or.b.clone(),
            }],
        ]
    }
}

impl<'ctx> MatchedComposite<'ctx> for UnlockLogic<Match<'ctx>> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool + 'ctx>> {
        vec![Box::new(|ul| {
            let valid = ul.is_valid_depth();
            if !valid {
                warn!(
                    "UnlockLogic filter: Invalid depth {} (expected 1-2)",
                    ul.or_tree.depth()
                );
            }
            valid
        })]
    }
}

impl SearchableComposite for UnlockLogic<Search> {
    type Hit<'ctx> = UnlockLogic<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        let and_context = AndGate::<Search>::context(driver, config)?;
        let or_tree_context = RecOr::<Search>::context(driver, config)?;
        let not_context = NotGate::<Search>::context(driver, config)?;

        Ok(and_context.merge(or_tree_context).merge(not_context))
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        info_span!("unlock_logic_query").in_scope(|| {
            info!("UnlockLogic::query starting");

            // Sub-queries with logging
            let and_matches = AndGate::<Search>::query(
                haystack_key,
                context,
                path.child("and".to_string()),
                config,
            );
            let or_tree_matches = RecOr::<Search>::query(
                haystack_key,
                context,
                path.child("or_tree".to_string()),
                config,
            );
            let not_matches = NotGate::<Search>::query(
                haystack_key,
                context,
                path.child("not".to_string()),
                config,
            );

            info!(
                and_count = and_matches.len(),
                or_tree_count = or_tree_matches.len(),
                not_count = not_matches.len(),
                "Sub-query results"
            );

            let mut candidates = Vec::new();
            let mut failed_reasons = Vec::new();

            // Cartesian product with detailed filtering
            for (and_gate, or_tree, not_gate) in iproduct!(
                and_matches.iter(),
                or_tree_matches.iter(),
                not_matches.iter()
            ) {
                let base_path = path.clone();
                let candidate = UnlockLogic {
                    path: base_path.clone(),
                    and: (*and_gate).clone(),
                    or_tree: (*or_tree).clone(),
                    not: (*not_gate).clone(),
                    write: crate::Wire::new(base_path.child("write".to_string())),
                    lock_status: crate::Wire::new(base_path.child("lock_status".to_string())),
                    scan_mode: crate::Wire::new(base_path.child("scan_mode".to_string())),
                    debug_unlocked: crate::Wire::new(base_path.child("debug_unlocked".to_string())),
                };

                // Depth filter
                if !candidate.is_valid_depth() {
                    failed_reasons.push(format!(
                        "Depth {} invalid for candidate at {:?}",
                        candidate.or_tree.depth(),
                        candidate.path
                    ));
                    continue;
                }

                // Connection validation with logging
                let conns = candidate.connections();
                let valid = candidate.validate_connections(conns.clone());
                if !valid {
                    debug!(
                        "Connection validation failed for candidate at {:?}: {:?}",
                        candidate.path, conns
                    );
                    failed_reasons.push(format!(
                        "Connections invalid for candidate at {:?}",
                        candidate.path
                    ));
                    continue;
                }

                // Other filters (e.g., from MatchedComposite)
                let filters_ok = candidate.other_filters().iter().all(|f| f(&candidate));
                if !filters_ok {
                    failed_reasons.push(format!(
                        "Filters failed for candidate at {:?}",
                        candidate.path
                    ));
                    continue;
                }

                info!("Valid candidate found at {:?}", candidate.path);
                candidates.push(candidate);
            }

            if !failed_reasons.is_empty() {
                warn!(
                    "UnlockLogic: {} candidates failed: {:?}",
                    failed_reasons.len(),
                    failed_reasons
                );
            }

            info!("UnlockLogic::query complete: {} matches", candidates.len());
            candidates
        })
    }
}
