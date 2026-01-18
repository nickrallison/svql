use std::collections::HashSet;

use common::{Config, ModuleConfig};
use driver::{Context, Driver, DriverKey};
use subgraph::{GraphIndex, cell::CellWrapper};

use crate::prelude::*;
use crate::traits::{MatchedComponent, SearchableComponent, kind};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RecOr<S>
where
    S: State,
{
    pub path: Instance,
    pub or: OrGate<S>,
    pub child: Option<Box<Self>>,
}

impl<S> RecOr<S>
where
    S: State,
{
    pub fn depth(&self) -> usize {
        1 + self.child.as_ref().map(|c| c.depth()).unwrap_or(0)
    }

    pub fn output(&self) -> &Wire<S> {
        &self.or.y
    }
}

impl<S> Hardware for RecOr<S>
where
    S: State,
{
    type State = S;

    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "RecOr"
    }

    fn children(&self) -> Vec<&dyn Hardware<State = Self::State>> {
        let mut children: Vec<&dyn Hardware<State = Self::State>> = vec![&self.or];
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
            children.push(child.or.report(&format!("[{}]", idx)));
            current = child.child.as_ref();
            idx += 1;
        }

        ReportNode {
            name: name.to_string(),
            type_name: "RecOr".to_string(),
            path: self.path.clone(),
            details: Some(format!("Depth: {}", self.depth())),
            source_loc: self.or.y.source(),
            children,
        }
    }
}

impl SearchableComponent for RecOr<Search> {
    type Kind = kind::Composite;
    type Match = RecOr<Match>;

    fn create_at(base_path: Instance) -> Self {
        Self::new(base_path)
    }

    fn build_context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        OrGate::<Search>::build_context(driver, config)
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
            "RecOr::execute: starting recursive OR gate search"
        );

        let haystack_index = context.get(key).unwrap().index();

        let or_query = OrGate::<Search>::instantiate(self.path.child("or"));
        let all_or_gates = or_query.execute(driver, context, key, config);

        tracing::event!(
            tracing::Level::INFO,
            "RecOr::execute: Found {} total OR gates in design",
            all_or_gates.len()
        );

        let mut current_layer: Vec<RecOr<Match>> = all_or_gates
            .iter()
            .map(|or_gate| RecOr {
                path: self.path.clone(),
                or: or_gate.clone(),
                child: None,
            })
            .collect();

        let mut all_results = current_layer.clone();
        let mut layer_num = 2;

        loop {
            let next_layer =
                build_next_layer(&self.path, &all_or_gates, &current_layer, haystack_index);

            if next_layer.is_empty() {
                break;
            }

            tracing::event!(
                tracing::Level::INFO,
                "RecOr::execute: Layer {} has {} matches",
                layer_num,
                next_layer.len()
            );

            all_results.extend(next_layer.iter().cloned());
            current_layer = next_layer;
            layer_num += 1;

            if let Some(max) = config.max_recursion_depth
                && layer_num > max
            {
                break;
            }
        }

        all_results
    }
}

impl MatchedComponent for RecOr<Match> {
    type Search = RecOr<Search>;
}

impl<S> Topology<S> for RecOr<S>
where
    S: State,
{
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        if let Some(ref child) = self.child {
            ctx.connect_any(&[
                (Some(&child.or.y), Some(&self.or.a)),
                (Some(&child.or.y), Some(&self.or.b)),
            ]);
        }
    }
}

impl RecOr<Search> {
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            or: OrGate::instantiate(path.child("or")),
            child: None,
        }
    }
}

impl<'ctx> RecOr<Match> {
    pub fn fanin_set(&self, haystack_index: &GraphIndex<'ctx>) -> HashSet<CellWrapper<'ctx>> {
        let mut all_cells = HashSet::new();
        self.collect_cell_ids(&mut all_cells);
        let mut fanin = HashSet::new();
        for cell_id in &all_cells {
            let Some(cell) = haystack_index.get_cell_by_id(*cell_id) else {
                continue;
            };
            if let Some(fanin_set) = haystack_index.fanin_set(&cell) {
                fanin.extend(fanin_set.iter().cloned());
            }
        }
        fanin
    }

    fn collect_cell_ids(&self, ids: &mut HashSet<usize>) {
        if let Some(ref info) = self.or.y.inner {
            ids.insert(info.id);
        }
        if let Some(ref child) = self.child {
            child.collect_cell_ids(ids);
        }
    }
}

fn rec_or_cell_ids(rec_or: &RecOr<Match>) -> Vec<usize> {
    let mut ids = Vec::new();
    if let Some(ref info) = rec_or.or.y.inner {
        ids.push(info.id);
    }

    if let Some(ref child) = rec_or.child {
        ids.extend(rec_or_cell_ids(child));
    }

    ids
}
fn validate_rec_candidate<'ctx>(
    path: &Instance,
    or_gate: &OrGate<Match>,
    prev: &RecOr<Match>,
    haystack_index: &GraphIndex<'ctx>,
) -> bool {
    let mut child = prev.clone();
    update_rec_or_path(&mut child, path.child("child"));

    let candidate = RecOr {
        path: path.clone(),
        or: or_gate.clone(),
        child: Some(Box::new(child)),
    };

    let mut builder = ConnectionBuilder {
        constraints: Vec::new(),
    };
    candidate.define_connections(&mut builder);

    builder.constraints.iter().all(|group| {
        group.iter().any(|(from, to)| match (from, to) {
            (Some(f), Some(t)) => validate_connection(f, t, haystack_index),
            _ => false,
        })
    })
}

fn build_next_layer<'ctx>(
    path: &Instance,
    all_or_gates: &[OrGate<Match>],
    prev_layer: &[RecOr<Match>],
    haystack_index: &GraphIndex<'ctx>,
) -> Vec<RecOr<Match>> {
    let start_time = std::time::Instant::now();

    let next_layer: Vec<_> = prev_layer
        .iter()
        .flat_map(|prev| {
            let top_info = prev.or.y.inner.as_ref()?;
            let top_wrapper = haystack_index.get_cell_by_id(top_info.id)?;
            let fanout = haystack_index.fanout_set(&top_wrapper)?;
            let contained_ids = rec_or_cell_ids(prev);

            Some(all_or_gates.iter().filter_map(move |or_gate| {
                let gate_info = or_gate.y.inner.as_ref()?;
                let gate_wrapper = haystack_index.get_cell_by_id(gate_info.id)?;

                let is_valid = fanout.contains(&gate_wrapper)
                    && !contained_ids.contains(&gate_info.id)
                    && validate_rec_candidate(path, or_gate, prev, haystack_index);

                is_valid.then(|| {
                    let mut child = prev.clone();
                    update_rec_or_path(&mut child, path.child("child"));
                    RecOr {
                        path: path.clone(),
                        or: or_gate.clone(),
                        child: Some(Box::new(child)),
                    }
                })
            }))
        })
        .flatten()
        .collect();

    tracing::event!(
        tracing::Level::INFO,
        "build_next_layer: Completed in {:?}, found {} matches",
        start_time.elapsed(),
        next_layer.len()
    );

    next_layer
}

fn update_rec_or_path<'ctx>(rec_or: &mut RecOr<Match>, new_path: Instance) {
    rec_or.path = new_path.clone();
    let or_path = new_path.child("or");
    rec_or.or.path = or_path.clone();
    rec_or.or.a.path = or_path.child("a");
    rec_or.or.b.path = or_path.child("b");
    rec_or.or.y.path = or_path.child("y");

    if let Some(ref mut child) = rec_or.child {
        update_rec_or_path(child, new_path.child("child"));
    }
}

// --- Dehydrate/Rehydrate implementations for DataFrame storage ---

use crate::session::{
    Dehydrate, Rehydrate, DehydratedResults, DehydratedRow, MatchRow, QuerySchema, 
    WireFieldDesc, RecursiveFieldDesc, RehydrateContext, SearchDehydrate, SessionError
};

/// Static schema for RecOr - stores the OR gate's wire cell IDs plus child reference
static REC_OR_RECURSIVE_FIELD: RecursiveFieldDesc = RecursiveFieldDesc { name: "child" };

impl Dehydrate for RecOr<Match> {
    const SCHEMA: QuerySchema = QuerySchema::with_recursive_child(
        "RecOr",
        &[
            WireFieldDesc { name: "or_a" },
            WireFieldDesc { name: "or_b" },
            WireFieldDesc { name: "or_y" },
        ],
        &[], // No external submodules, just the recursive child
        &REC_OR_RECURSIVE_FIELD,
    );
    
    fn dehydrate(&self) -> DehydratedRow {
        DehydratedRow::new(self.path.to_string())
            .with_wire("or_a", self.or.a.inner.as_ref().map(|c| c.id as u32))
            .with_wire("or_b", self.or.b.inner.as_ref().map(|c| c.id as u32))
            .with_wire("or_y", self.or.y.inner.as_ref().map(|c| c.id as u32))
            .with_depth(self.depth() as u32)
            // Note: child_idx must be set by the caller when building the results table
            // because it requires knowing the index of the child row
    }
}

impl Rehydrate for RecOr<Match> {
    const TYPE_NAME: &'static str = "RecOr";
    
    fn rehydrate(
        row: &MatchRow,
        ctx: &RehydrateContext<'_>,
    ) -> Result<Self, SessionError> {
        let path = Instance::from_path(&row.path);
        let or_path = path.child("or");
        
        // Rehydrate the OR gate
        let or = OrGate {
            path: or_path.clone(),
            a: ctx.rehydrate_wire(or_path.child("a"), row.wire("or_a")),
            b: ctx.rehydrate_wire(or_path.child("b"), row.wire("or_b")),
            y: ctx.rehydrate_wire(or_path.child("or_y"), row.wire("or_y")),
        };
        
        // Recursively rehydrate child if present
        let child = if let Some(child_idx) = row.child() {
            let child_row = ctx.get_match_row(Self::TYPE_NAME, child_idx)
                .ok_or_else(|| SessionError::InvalidMatchIndex(child_idx))?;
            Some(Box::new(Self::rehydrate(&child_row, ctx)?))
        } else {
            None
        };
        
        Ok(RecOr { path, or, child })
    }
}

impl SearchDehydrate for RecOr<Search> {
    const MATCH_SCHEMA: QuerySchema = <RecOr<Match> as Dehydrate>::SCHEMA;
    
    fn execute_dehydrated(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
        results: &mut DehydratedResults,
    ) -> Vec<u32> {
        let haystack_index = context.get(key).unwrap().index();

        // Get all OR gates (dehydrated)
        let or_query = OrGate::<Search>::instantiate(self.path.child("or"));
        let all_or_indices = or_query.execute_dehydrated(driver, context, key, config, results);
        
        // Get the OR gate table from results
        let or_table = results.tables.get("OrGate").cloned().unwrap_or_default();
        
        // Build layer 1: single OR gates (no child)
        let mut current_layer: Vec<(u32, u32)> = Vec::new(); // (rec_or_idx, or_cell_id)
        
        for &or_idx in &all_or_indices {
            if let Some(or_row) = or_table.get(or_idx as usize) {
                let or_cell_id = or_row.wire("y").unwrap_or(u32::MAX);
                let rec_or_row = DehydratedRow::new(self.path.to_string())
                    .with_wire("or_a", or_row.wire("a"))
                    .with_wire("or_b", or_row.wire("b"))
                    .with_wire("or_y", or_row.wire("y"))
                    .with_depth(1)
                    .with_child(None);
                let rec_or_idx = results.push("RecOr", rec_or_row);
                current_layer.push((rec_or_idx, or_cell_id));
            }
        }
        
        let mut all_rec_or_indices: Vec<u32> = current_layer.iter().map(|(idx, _)| *idx).collect();
        let mut layer_num = 2u32;
        
        // Build subsequent layers
        loop {
            let mut next_layer: Vec<(u32, u32)> = Vec::new();
            
            for &or_idx in &all_or_indices {
                if let Some(or_row) = or_table.get(or_idx as usize) {
                    let or_a = or_row.wire("a");
                    let or_b = or_row.wire("b");
                    
                    // Check if any current layer output connects to this OR gate's inputs
                    for &(child_rec_or_idx, child_y_cell_id) in &current_layer {
                        let child_connects = [or_a, or_b].iter().any(|input| {
                            if let Some(input_id) = input {
                                // Check connectivity: child_y -> or_input
                                if let (Some(from_cell), Some(to_cell)) = (
                                    haystack_index.get_cell_by_id(child_y_cell_id as usize),
                                    haystack_index.get_cell_by_id(*input_id as usize)
                                ) {
                                    haystack_index.fanout_set(&from_cell)
                                        .map(|fanout| fanout.contains(&to_cell))
                                        .unwrap_or(false)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });
                        
                        if child_connects {
                            let rec_or_row = DehydratedRow::new(self.path.to_string())
                                .with_wire("or_a", or_a)
                                .with_wire("or_b", or_b)
                                .with_wire("or_y", or_row.wire("y"))
                                .with_depth(layer_num)
                                .with_child(Some(child_rec_or_idx));
                            let rec_or_idx = results.push("RecOr", rec_or_row);
                            let or_cell_id = or_row.wire("y").unwrap_or(u32::MAX);
                            next_layer.push((rec_or_idx, or_cell_id));
                        }
                    }
                }
            }
            
            if next_layer.is_empty() {
                break;
            }
            
            all_rec_or_indices.extend(next_layer.iter().map(|(idx, _)| *idx));
            current_layer = next_layer;
            layer_num += 1;
            
            if let Some(max) = config.max_recursion_depth {
                if layer_num > max as u32 {
                    break;
                }
            }
        }
        
        all_rec_or_indices
    }
}
