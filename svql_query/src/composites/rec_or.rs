use std::collections::HashSet;
use std::sync::Arc;

use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::{GraphIndex, cell::CellWrapper};

use crate::{
    Match, Search, State, Wire,
    instance::Instance,
    primitives::or::OrGate,
    traits::{
        Component, ConnectionBuilder, PlannedQuery, Query, Searchable, Topology,
        validate_connection,
    },
};

#[derive(Debug, Clone)]
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

impl<S> Component<S> for RecOr<S>
where
    S: State,
{
    fn path(&self) -> &Instance {
        &self.path
    }

    fn type_name(&self) -> &'static str {
        "RecOr"
    }

    fn children(&self) -> Vec<&dyn Component<S>> {
        let mut kids: Vec<&dyn Component<S>> = vec![&self.or];
        if let Some(c) = &self.child {
            kids.push(c.as_ref());
        }
        kids
    }

    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        if let Some(port) = self.or.find_port(p) {
            return Some(port);
        }
        if let Some(ref child) = self.child {
            if let Some(port) = child.find_port(p) {
                return Some(port);
            }
        }
        None
    }

    fn find_port_inner(&self, _rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        None
    }
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

impl Searchable for RecOr<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self::new(base_path)
    }
}

impl RecOr<Search> {
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            or: OrGate::new(path.child("or")),
            child: None,
        }
    }

    pub fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        OrGate::<Search>::context(driver, config)
    }
}

impl Query for RecOr<Search> {
    type Matched<'a> = RecOr<Match<'a>>;

    fn query<'a>(
        &self,
        driver: &Driver,
        context: &'a Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Matched<'a>> {
        tracing::event!(
            tracing::Level::INFO,
            "RecOr::query: starting recursive OR gate search"
        );

        let haystack_index = context.get(key).unwrap().index();

        let or_query = OrGate::<Search>::instantiate(self.path.child("or"));
        let all_or_gates = or_query.query(driver, context, key, config);

        tracing::event!(
            tracing::Level::INFO,
            "RecOr::query: Found {} total OR gates in design",
            all_or_gates.len()
        );

        let mut current_layer: Vec<RecOr<Match<'a>>> = all_or_gates
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
                "RecOr::query: Layer {} has {} matches",
                layer_num,
                next_layer.len()
            );

            all_results.extend(next_layer.iter().cloned());
            current_layer = next_layer;
            layer_num += 1;

            if let Some(max) = config.max_recursion_depth {
                if layer_num > max {
                    break;
                }
            }
        }

        all_results
    }
}

impl PlannedQuery for RecOr<Search> {
    fn to_ir(&self, config: &::svql_common::Config) -> ::svql_query::ir::LogicalPlan {
        if let Some(ref child) = self.child {
            let inputs = vec![
                Box::new(self.or.to_ir(config)),
                Box::new(child.to_ir(config)),
            ];

            let mut builder = ::svql_query::traits::ConnectionBuilder {
                constraints: Vec::new(),
            };
            self.define_connections(&mut builder);

            let mut join_constraints = Vec::new();

            let map_wire =
                |wire: &::svql_query::Wire<::svql_query::Search>| -> Option<(usize, usize)> {
                    let wire_path = wire.path();
                    if wire_path.starts_with(self.or.path()) {
                        let rel = wire_path.relative(self.or.path());
                        if let Some(col) = self.or.get_column_index(rel) {
                            return Some((0, col));
                        }
                    }
                    if wire_path.starts_with(child.path()) {
                        let rel = wire_path.relative(child.path());
                        if let Some(col) = child.get_column_index(rel) {
                            return Some((1, col));
                        }
                    }
                    None
                };

            for group in builder.constraints {
                let mut or_group = Vec::new();
                for (from_opt, to_opt) in group {
                    if let (Some(from), Some(to)) = (from_opt, to_opt) {
                        if let (Some(src), Some(dst)) = (map_wire(from), map_wire(to)) {
                            or_group.push((src, dst));
                        }
                    }
                }
                if !or_group.is_empty() {
                    if or_group.len() == 1 {
                        join_constraints.push(::svql_query::ir::JoinConstraint::Eq(
                            or_group[0].0,
                            or_group[0].1,
                        ));
                    } else {
                        join_constraints.push(::svql_query::ir::JoinConstraint::Or(or_group));
                    }
                }
            }

            ::svql_query::ir::LogicalPlan::Join {
                inputs,
                constraints: join_constraints,
                schema: self.expected_schema(),
            }
        } else {
            self.or.to_ir(config)
        }
    }

    fn expected_schema(&self) -> ::svql_query::ir::Schema {
        let mut schema = ::svql_query::ir::Schema {
            columns: Vec::new(),
        };
        schema.columns.extend(self.or.expected_schema().columns);
        if let Some(ref child) = self.child {
            schema.columns.extend(child.expected_schema().columns);
        }
        schema
    }

    fn get_column_index(&self, rel_path: &[std::sync::Arc<str>]) -> Option<usize> {
        let next = match rel_path.first() {
            Some(arc_str) => arc_str.as_ref(),
            None => return None,
        };
        let tail = &rel_path[1..];
        match next {
            "or" => {
                let sub_idx = self.or.get_column_index(tail)?;
                Some(0 + sub_idx)
            }
            "child" => {
                if let Some(ref child) = self.child {
                    let sub_idx = child.get_column_index(tail)?;
                    let offset = self.or.expected_schema().columns.len();
                    Some(offset + sub_idx)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn reconstruct<'a>(
        &self,
        cursor: &mut ::svql_query::ir::ResultCursor<'a>,
    ) -> Self::Matched<'a> {
        RecOr {
            path: self.path.clone(),
            or: self.or.reconstruct(cursor),
            child: self.child.as_ref().map(|c| Box::new(c.reconstruct(cursor))),
        }
    }
}

impl<'ctx> RecOr<Match<'ctx>> {
    pub fn fanin_set(&self, haystack_index: &GraphIndex<'ctx>) -> HashSet<CellWrapper<'ctx>> {
        let mut all_cells = HashSet::new();
        self.collect_cells(&mut all_cells);
        let mut fanin = HashSet::new();
        for cell in &all_cells {
            if let Some(fanin_set) = haystack_index.fanin_set(cell) {
                fanin.extend(fanin_set.iter().cloned());
            }
        }
        fanin
    }

    fn collect_cells(&self, cells: &mut HashSet<CellWrapper<'ctx>>) {
        let or_cell = &self.or.y.inner;
        cells.insert(or_cell.clone());
        if let Some(ref child) = self.child {
            child.collect_cells(cells);
        }
    }
}

fn rec_or_cells<'a, 'ctx>(rec_or: &'a RecOr<Match<'ctx>>) -> Vec<&'a CellWrapper<'ctx>> {
    let mut cells = Vec::new();
    let or_cell = &rec_or.or.y.inner;
    cells.push(or_cell);

    if let Some(ref child) = rec_or.child {
        cells.extend(rec_or_cells(child));
    }

    cells
}

fn build_next_layer<'ctx>(
    path: &Instance,
    all_or_gates: &[OrGate<Match<'ctx>>],
    prev_layer: &[RecOr<Match<'ctx>>],
    haystack_index: &GraphIndex<'ctx>,
) -> Vec<RecOr<Match<'ctx>>> {
    let mut next_layer = Vec::new();

    for prev in prev_layer {
        let top_or_cell = &prev.or.y.inner;
        let fanout = haystack_index
            .fanout_set(top_or_cell)
            .expect("Fanout Not found for cell");
        let contained_cells = rec_or_cells(prev);

        for or_gate in all_or_gates {
            let cell = &or_gate.y.inner;

            if !fanout.contains(cell) || contained_cells.contains(&cell) {
                continue;
            }

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

            if valid {
                next_layer.push(candidate);
            }
        }
    }

    next_layer
}

fn update_rec_or_path<'ctx>(rec_or: &mut RecOr<Match<'ctx>>, new_path: Instance) {
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
