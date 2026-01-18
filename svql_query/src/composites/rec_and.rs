use common::{Config, ModuleConfig};
use driver::{Context, Driver, DriverKey};
use subgraph::GraphIndex;

use crate::prelude::*;
use crate::traits::{MatchedComponent, SearchableComponent, kind};

#[derive(Debug, Clone)]
pub struct RecAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub and: AndGate<S>,
    pub child: Option<Box<Self>>,
}

impl<S> RecAnd<S>
where
    S: State,
{
    /// Get the depth of this recursive structure (1 for just an AND gate, 2+ for nested)
    pub fn depth(&self) -> usize {
        1 + self.child.as_ref().map(|c| c.depth()).unwrap_or(0)
    }

    /// Get the output wire of the top-level AND gate
    pub fn output(&self) -> &Wire<S> {
        &self.and.y
    }
}

impl<S> Hardware for RecAnd<S>
where
    S: State,
{
    type State = S;

    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "RecAnd"
    }

    fn children(&self) -> Vec<&dyn Hardware<State = Self::State>> {
        let mut children: Vec<&dyn Hardware<State = Self::State>> = vec![&self.and];
        if let Some(child) = &self.child {
            children.push(child.as_ref());
        }
        children
    }

    fn report(&self, name: &str) -> ReportNode {
        let mut children = Vec::new();
        let mut current = self.child.as_ref();
        let mut idx = 0;

        while let Some(child) = current {
            children.push(child.and.report(&format!("[{}]", idx)));
            current = child.child.as_ref();
            idx += 1;
        }

        ReportNode {
            name: name.to_string(),
            type_name: "RecAnd".to_string(),
            path: self.path.clone(),
            details: Some(format!("Depth: {}", self.depth())),
            source_loc: self.and.y.source(),
            children,
        }
    }
}

impl SearchableComponent for RecAnd<Search> {
    type Kind = kind::Composite;
    type Match = RecAnd<Match>;

    fn create_at(base_path: Instance) -> Self {
        Self::new(base_path)
    }

    fn build_context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        AndGate::<Search>::build_context(driver, config)
    }

    fn execute_search(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Match> {
        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::execute: starting recursive AND gate search"
        );

        let haystack_index = context.get(key).unwrap().index();

        let and_query = AndGate::<Search>::instantiate(self.path.child("and"));
        let all_and_gates = and_query.execute(driver, context, key, config);

        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::execute: Found {} total AND gates in design",
            all_and_gates.len()
        );

        let mut current_layer: Vec<RecAnd<Match>> = all_and_gates
            .iter()
            .map(|and_gate| RecAnd {
                path: self.path.clone(),
                and: and_gate.clone(),
                child: None,
            })
            .collect();

        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::execute: Layer 1 (base case) has {} matches",
            current_layer.len()
        );

        let mut all_results = current_layer.clone();
        let mut layer_num = 2;

        loop {
            let next_layer =
                build_next_layer(&self.path, &all_and_gates, &current_layer, haystack_index);

            if next_layer.is_empty() {
                tracing::event!(
                    tracing::Level::INFO,
                    "RecAnd::execute: No more matches at layer {}, stopping",
                    layer_num
                );
                break;
            }

            tracing::event!(
                tracing::Level::INFO,
                "RecAnd::execute: Layer {} has {} matches",
                layer_num,
                next_layer.len()
            );

            all_results.extend(next_layer.iter().cloned());
            current_layer = next_layer;
            layer_num += 1;

            if let Some(max) = config.max_recursion_depth
                && layer_num > max {
                    tracing::event!(
                        tracing::Level::INFO,
                        "RecAnd::execute: Reached max recursion depth of {}, stopping",
                        max
                    );
                    break;
                }
        }

        tracing::event!(
            tracing::Level::INFO,
            "RecAnd::execute: Total {} matches across {} layers",
            all_results.len(),
            layer_num - 1
        );

        all_results
    }
}

impl MatchedComponent for RecAnd<Match> {
    type Search = RecAnd<Search>;
}

impl<S> Topology<S> for RecAnd<S>
where
    S: State,
{
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        if let Some(ref child) = self.child {
            ctx.connect_any(&[
                (Some(&child.and.y), Some(&self.and.a)),
                (Some(&child.and.y), Some(&self.and.b)),
            ]);
        }
    }
}

impl RecAnd<Search> {
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            and: AndGate::instantiate(path.child("and")),
            child: None,
        }
    }
}

fn rec_and_cell_ids(rec_and: &RecAnd<Match>) -> Vec<usize> {
    let mut ids = Vec::new();
    if let Some(ref info) = rec_and.and.y.inner {
        ids.push(info.id);
    }

    if let Some(ref child) = rec_and.child {
        ids.extend(rec_and_cell_ids(child));
    }

    ids
}

fn validate_rec_candidate<'ctx>(
    path: &Instance,
    and_gate: &AndGate<Match>,
    prev: &RecAnd<Match>,
    haystack_index: &GraphIndex<'ctx>,
) -> bool {
    // We create a temporary candidate to run the topology check
    let mut child = prev.clone();
    update_rec_and_path(&mut child, path.child("child"));

    let candidate = RecAnd {
        path: path.clone(),
        and: and_gate.clone(),
        child: Some(Box::new(child)),
    };

    let mut builder = ConnectionBuilder {
        constraints: Vec::new(),
    };
    candidate.define_connections(&mut builder);

    // Ensure all connection groups have at least one valid physical connection
    builder.constraints.iter().all(|group| {
        group.iter().any(|(from, to)| match (from, to) {
            (Some(f), Some(t)) => validate_connection(f, t, haystack_index),
            _ => false,
        })
    })
}

fn build_next_layer<'ctx>(
    path: &Instance,
    all_and_gates: &[AndGate<Match>],
    prev_layer: &[RecAnd<Match>],
    haystack_index: &GraphIndex<'ctx>,
) -> Vec<RecAnd<Match>> {
    prev_layer
        .iter()
        .flat_map(|prev| {
            let top_info = prev.and.y.inner.as_ref()?;
            let top_wrapper = haystack_index.get_cell_by_id(top_info.id)?;
            let fanout = haystack_index.fanout_set(&top_wrapper)?;
            let contained_ids = rec_and_cell_ids(prev);

            Some(all_and_gates.iter().filter_map(move |and_gate| {
                let gate_info = and_gate.y.inner.as_ref()?;
                let gate_wrapper = haystack_index.get_cell_by_id(gate_info.id)?;

                let is_valid = fanout.contains(&gate_wrapper)
                    && !contained_ids.contains(&gate_info.id)
                    && validate_rec_candidate(path, and_gate, prev, haystack_index);

                is_valid.then(|| {
                    let mut child = prev.clone();
                    update_rec_and_path(&mut child, path.child("child"));
                    RecAnd {
                        path: path.clone(),
                        and: and_gate.clone(),
                        child: Some(Box::new(child)),
                    }
                })
            }))
        })
        .flatten()
        .collect()
}

fn update_rec_and_path<'ctx>(rec_and: &mut RecAnd<Match>, new_path: Instance) {
    rec_and.path = new_path.clone();
    let and_path = new_path.child("and");
    rec_and.and.path = and_path.clone();
    rec_and.and.a.path = and_path.child("a");
    rec_and.and.b.path = and_path.child("b");
    rec_and.and.y.path = and_path.child("y");

    if let Some(ref mut child) = rec_and.child {
        update_rec_and_path(child, new_path.child("child"));
    }
}

// --- Dehydrate/Rehydrate implementations for DataFrame storage ---

use crate::session::{
    Dehydrate, Rehydrate, DehydratedRow, MatchRow, QuerySchema, 
    WireFieldDesc, RecursiveFieldDesc, RehydrateContext, SessionError
};

/// Static schema for RecAnd - stores the AND gate's wire cell IDs plus child reference
static REC_AND_RECURSIVE_FIELD: RecursiveFieldDesc = RecursiveFieldDesc { name: "child" };

impl Dehydrate for RecAnd<Match> {
    const SCHEMA: QuerySchema = QuerySchema::with_recursive_child(
        "RecAnd",
        &[
            WireFieldDesc { name: "and_a" },
            WireFieldDesc { name: "and_b" },
            WireFieldDesc { name: "and_y" },
        ],
        &[], // No external submodules, just the recursive child
        &REC_AND_RECURSIVE_FIELD,
    );
    
    fn dehydrate(&self) -> DehydratedRow {
        DehydratedRow::new(self.path.to_string())
            .with_wire("and_a", self.and.a.inner.as_ref().map(|c| c.id as u32))
            .with_wire("and_b", self.and.b.inner.as_ref().map(|c| c.id as u32))
            .with_wire("and_y", self.and.y.inner.as_ref().map(|c| c.id as u32))
            .with_depth(self.depth() as u32)
            // Note: child_idx must be set by the caller when building the results table
            // because it requires knowing the index of the child row
    }
}

impl Rehydrate for RecAnd<Match> {
    const TYPE_NAME: &'static str = "RecAnd";
    
    fn rehydrate(
        row: &MatchRow,
        ctx: &RehydrateContext<'_>,
    ) -> Result<Self, SessionError> {
        let path = Instance::from_path(&row.path);
        let and_path = path.child("and");
        
        // Rehydrate the AND gate
        let and = AndGate {
            path: and_path.clone(),
            a: ctx.rehydrate_wire(and_path.child("a"), row.wire("and_a")),
            b: ctx.rehydrate_wire(and_path.child("b"), row.wire("and_b")),
            y: ctx.rehydrate_wire(and_path.child("and_y"), row.wire("and_y")),
        };
        
        // Recursively rehydrate child if present
        let child = if let Some(child_idx) = row.child() {
            let child_row = ctx.get_match_row(Self::TYPE_NAME, child_idx)
                .ok_or_else(|| SessionError::InvalidMatchIndex(child_idx))?;
            Some(Box::new(Self::rehydrate(&child_row, ctx)?))
        } else {
            None
        };
        
        Ok(RecAnd { path, and, child })
    }
}
