use svql_driver::{Driver, DriverKey};

use crate::{
    prelude::{ColumnDef, ColumnKind, QueryError, Table},
    selector::Selector,
    session::{AnyTable, ColumnEntry, EntryArray, ExecInfo, ExecutionContext, Row, Store},
    traits::{Component, PatternInternal, kind, search_table_any},
};

pub trait Composite: Sized + Component<Kind = kind::Composite> + Send + Sync + 'static {
    /// Schema definition for DataFrame storage.
    const DEFS: &'static [ColumnDef];

    /// Size of the schema (number of columns).w
    const SCHEMA_SIZE: usize = Self::DEFS.len();

    const CONNECTIONS: Connections;

    const ALIASES: &'static [(&'static str, Endpoint)];

    const DEPENDANCIES: &'static [&'static ExecInfo];

    /// Access the smart Schema wrapper.
    fn schema() -> &'static crate::session::PatternSchema;

    /// Core logic to compose sub-matches into a result table.
    ///
    /// Performs incremental joins: each submodule is joined one at a time, with
    /// invalid matches filtered out after each step. The join order is currently
    /// determined by the order of submodule indices, but the implementation is
    /// designed to support arbitrary ordering for future optimization.
    ///
    /// # Arguments
    /// * `ctx` - The execution context (access to driver, config, etc).
    /// * `dep_tables` - A slice aligned 1:1 with `schema.submodules`.
    fn compose(
        ctx: &ExecutionContext,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
    ) -> Result<Table<Self>, QueryError> {
        let schema = Self::schema();
        let sub_indices = &schema.submodules;

        // Early exit: any required (non-nullable) empty table means no results
        for (i, &col_idx) in sub_indices.iter().enumerate() {
            if dep_tables[i].is_empty() && !schema.column(col_idx).nullable {
                return Table::new(vec![]);
            }
        }

        // Build join order - currently sequential, but decoupled for future optimization
        let join_order: Vec<usize> = (0..sub_indices.len()).collect();

        // Seed with partial entries from the first submodule
        let first_idx = join_order[0];
        let first_table = dep_tables[first_idx];
        let mut entries: Vec<EntryArray> = (0..first_table.len() as u64)
            .map(|row_idx| Self::create_partial_entry(sub_indices, first_idx, row_idx))
            .collect();

        // Incrementally join each remaining submodule, filtering after each
        for &join_idx in &join_order[1..] {
            let table = dep_tables[join_idx];
            entries = Self::join_and_filter(entries, join_idx, table, sub_indices, dep_tables, ctx);

            if entries.is_empty() {
                return Table::new(vec![]);
            }
        }

        let final_entries: Vec<EntryArray> = entries
            .into_iter()
            .map(|mut entry| {
                // Create temp row for resolution
                let row = Row::<Self> {
                    idx: 0,
                    entry_array: entry.clone(),
                    _marker: std::marker::PhantomData,
                };

                for (alias_col_name, endpoint) in Self::ALIASES {
                    // Resolve the deep path
                    let val =
                        if let Some(cell_id) = endpoint.resolve_endpoint(&row, dep_tables, ctx) {
                            ColumnEntry::Cell { id: Some(cell_id) }
                        } else {
                            ColumnEntry::Cell { id: None }
                        };

                    // Write to local column (requires looking up index, but this is done once per result row, not per candidate pair)
                    if let Some(idx) = Self::schema().index_of(alias_col_name) {
                        entry.entries[idx] = val;
                    }
                }
                entry
            })
            .collect();

        Table::new(final_entries)
    }

    /// Create a partial entry array with a single submodule slot populated.
    fn create_partial_entry(sub_indices: &[usize], join_idx: usize, row_idx: u64) -> EntryArray {
        let mut entries = vec![ColumnEntry::Metadata { id: None }; Self::SCHEMA_SIZE];
        entries[sub_indices[join_idx]] = ColumnEntry::Sub { id: Some(row_idx) };
        EntryArray::new(entries)
    }

    /// Join existing partial entries with a new submodule table, filtering invalid combinations.
    fn join_and_filter(
        entries: Vec<EntryArray>,
        join_idx: usize,
        table: &(dyn AnyTable + Send + Sync),
        sub_indices: &[usize],
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
        ctx: &ExecutionContext,
    ) -> Vec<EntryArray> {
        let col_idx = sub_indices[join_idx];

        entries
            .into_iter()
            .flat_map(|entry| {
                (0..table.len() as u64).filter_map(move |new_row_idx| {
                    let mut candidate = entry.clone();
                    candidate.entries[col_idx] = ColumnEntry::Sub {
                        id: Some(new_row_idx),
                    };

                    let row = Row::<Self> {
                        idx: 0,
                        entry_array: candidate.clone(),
                        _marker: std::marker::PhantomData,
                    };

                    // FIX: Pass ctx to validate
                    Self::validate(&row, dep_tables, ctx).then_some(candidate)
                })
            })
            .collect()
    }

    /// Validates a row against connectivity constraints.
    ///
    /// Refactored to accept `dep_tables` directly, allowing validation during
    /// the `compose` phase before the final Table is fully built.
    fn validate(
        row: &Row<Self>,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
        ctx: &ExecutionContext,
    ) -> bool {
        let driver = ctx.driver();
        let key = &ctx.design_key();

        // 1. Access Graph
        // Note: In a hot loop, you might want to hoist this graph lookup out of validate
        // and pass the `GraphIndex` directly, but this signature matches the trait.
        let design_handle =
            match driver.get_design(key, &svql_common::Config::default().haystack_options) {
                Ok(d) => d,
                Err(_) => return false,
            };
        let _graph = design_handle.index();

        // 2. Iterate CNF Constraints
        for group in Self::CONNECTIONS.connections {
            let mut group_satisfied = false;
            let mut group_has_unresolvable = false;

            for connection in *group {
                // Pass the pre-fetched tables to resolve_endpoint
                let src_wire = connection.from.resolve_endpoint(row, dep_tables, ctx);
                let dst_wire = connection.to.resolve_endpoint(row, dep_tables, ctx);

                match (src_wire, dst_wire) {
                    (Some(s), Some(d)) => {
                        // Check physical connectivity in the netlist graph
                        // if graph.is_connected(s, d) {
                        //     group_satisfied = true;
                        //     break;
                        // }
                        if s == d {
                            group_satisfied = true;
                            break;
                        }
                    }
                    _ => {
                        // If we couldn't resolve a wire (e.g. partial match, nullable column),
                        // we can't fail the group yet.
                        group_has_unresolvable = true;
                    }
                }
            }

            if group_satisfied {
                continue;
            }
            // If the group is not satisfied and we have no "maybe" wires, the row is invalid.
            if !group_has_unresolvable {
                return false;
            }
        }

        true
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized;

    // the rest is tbd
    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Composite> + Send + Sync + 'static;
}

impl<T> PatternInternal<kind::Composite> for T
where
    T: Composite + Component<Kind = kind::Composite> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = T::DEFS;

    const SCHEMA_SIZE: usize = T::SCHEMA_SIZE;

    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: |ctx| {
            search_table_any::<T>(ctx, <T as PatternInternal<kind::Composite>>::search_table)
        },
        nested_dependancies: T::DEPENDANCIES,
    };

    fn schema() -> &'static crate::session::PatternSchema {
        T::schema()
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        <T as Composite>::preload_driver(driver, design_key, config)
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        let mut dep_tables = Vec::new();

        for sub_idx in T::schema().submodules.iter() {
            let tid = T::schema()
                .column(*sub_idx)
                .as_submodule()
                .expect("Idx should point to submodule");
            let table = ctx.get_any_table(tid).ok_or_else(|| {
                QueryError::MissingDependency(format!(
                    "TypeId {:?}, Col: {}",
                    tid,
                    T::schema().column(*sub_idx).name
                ))
            })?;
            dep_tables.push(table);
        }

        // 3. Hand off to the specific implementation to do the join/filter
        T::compose(ctx, &dep_tables)
    }

    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Composite> + Send + Sync + 'static,
    {
        <T as Composite>::rehydrate(row, store, driver, key)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Endpoint {
    pub selector: Selector,
}

impl Endpoint {
    /// Resolves an endpoint to a physical Cell ID.
    pub fn resolve_endpoint<T>(
        &self,
        row: &Row<T>,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
        ctx: &ExecutionContext,
    ) -> Option<u64>
    where
        T: Composite + Component,
    {
        let path = self.selector.path;
        if path.is_empty() {
            return None;
        }

        // 1. Resolve Head (Local Schema Lookup)
        let head_name = path[0];

        // Note: This string lookup happens per-row.
        // See "Aliases" section below for how to optimize this.
        let col_idx = T::schema().index_of(head_name)?;
        let col_def = T::schema().column(col_idx);
        let entry = row.entry_array.entries.get(col_idx)?;

        // 2. Check if we are done (Path length 1)
        if path.len() == 1 {
            return match entry {
                ColumnEntry::Cell { id } => *id,
                _ => None, // Path ended but it wasn't a cell
            };
        }

        // 3. Prepare for Traversal (Head must be a Submodule)
        let (mut current_row_idx, mut current_type_id) = match (entry, &col_def.kind) {
            (ColumnEntry::Sub { id: Some(id) }, ColumnKind::Sub(tid)) => (*id, *tid),
            _ => return None,
        };

        // Optimization: Check if the head is in dep_tables (immediate child)
        let mut next_table: Option<&(dyn AnyTable + Send + Sync)> = None;

        // Map column index to dependency index
        if let Some(dep_idx) = T::schema().submodules.iter().position(|&i| i == col_idx) {
            next_table = Some(dep_tables[dep_idx]);
        }

        // 4. Traverse Tail
        let mut path_iter = path[1..].iter().peekable();

        while let Some(segment_name) = path_iter.next() {
            // Fetch table (from optimization or global context)
            let table = match next_table {
                Some(t) => t,
                None => ctx.get_any_table(current_type_id)?,
            };

            if path_iter.peek().is_none() {
                // Final segment: Must be a Cell
                return table.get_cell_id(current_row_idx as usize, segment_name);
            } else {
                // Intermediate segment: Must be a Submodule
                let (next_idx, next_tid) =
                    table.get_sub_ref(current_row_idx as usize, segment_name)?;
                current_row_idx = next_idx;
                current_type_id = next_tid;
                next_table = None; // Reset optimization
            }
        }

        None
    }
}

pub struct Connection {
    pub from: Endpoint,
    pub to: Endpoint,
}

pub struct Connections {
    // in CNF form (helpful for some patterns like either (y -> a or y -> b) and (z -> c or z -> d))
    pub connections: &'static [&'static [Connection]],
}

#[allow(unused)]
mod test {

    use crate::{
        Wire,
        prelude::ColumnKind,
        traits::{Netlist, Pattern},
    };

    use super::*;

    use svql_common::Dedupe;
    use svql_query::query_test;

    #[derive(Debug, Clone)]
    struct AndGate {
        a: Wire,
        b: Wire,
        y: Wire,
    }

    impl Netlist for AndGate {
        const MODULE_NAME: &'static str = "and_gate";
        const FILE_PATH: &'static str = "examples/fixtures/basic/and/verilog/and_gate.v";
        const DEFS: &'static [ColumnDef] = &[
            ColumnDef::new("a", ColumnKind::Cell, false),
            ColumnDef::new("b", ColumnKind::Cell, false),
            ColumnDef::new("y", ColumnKind::Cell, false),
        ];

        fn schema() -> &'static crate::session::PatternSchema {
            static INSTANCE: std::sync::OnceLock<crate::session::PatternSchema> =
                std::sync::OnceLock::new();
            INSTANCE.get_or_init(|| crate::session::PatternSchema::new(<Self as Netlist>::DEFS))
        }

        fn rehydrate<'a>(
            row: &Row<Self>,
            store: &Store,
            driver: &Driver,
            key: &DriverKey,
        ) -> Option<Self>
        where
            Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static,
        {
            let a_id = row.wire("a")?;
            let b_id = row.wire("b")?;
            let y_id = row.wire("y")?;

            let and_gate = AndGate {
                a: a_id,
                b: b_id,
                y: y_id,
            };

            Some(and_gate)
        }
    }

    impl Component for AndGate {
        type Kind = kind::Netlist;
    }

    #[derive(Debug, Clone)]
    pub struct And2Gates {
        and1: AndGate,
        and2: AndGate,
    }

    impl Composite for And2Gates {
        const DEFS: &'static [ColumnDef] = &[
            ColumnDef::sub::<AndGate>("and1"),
            ColumnDef::sub::<AndGate>("and2"),
        ];

        const ALIASES: &'static [(&'static str, Endpoint)] = &[];

        fn schema() -> &'static crate::session::PatternSchema {
            static INSTANCE: std::sync::OnceLock<crate::session::PatternSchema> =
                std::sync::OnceLock::new();
            INSTANCE.get_or_init(|| crate::session::PatternSchema::new(<Self as Composite>::DEFS))
        }

        fn rehydrate<'a>(
            row: &Row<Self>,
            store: &Store,
            driver: &Driver,
            key: &DriverKey,
        ) -> Option<Self>
        where
            Self: Component + PatternInternal<kind::Composite> + Send + Sync + 'static,
        {
            todo!()
        }

        const CONNECTIONS: Connections = {
            let conns: &'static [&'static [Connection]] = &[&[
                // Connect and1.y -> and2.a
                Connection {
                    from: Endpoint {
                        selector: Selector::new(&["and1", "y"]),
                    },
                    to: Endpoint {
                        selector: Selector::new(&["and2", "a"]),
                    },
                },
                Connection {
                    from: Endpoint {
                        selector: Selector::new(&["and1", "y"]),
                    },
                    to: Endpoint {
                        selector: Selector::new(&["and2", "b"]),
                    },
                },
            ]];
            Connections { connections: conns }
        };

        const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AndGate as Pattern>::EXEC_INFO];

        fn preload_driver(
            driver: &Driver,
            design_key: &DriverKey,
            config: &svql_common::Config,
        ) -> Result<(), Box<dyn std::error::Error>>
        where
            Self: Sized,
        {
            <AndGate as Pattern>::preload_driver(driver, design_key, config)
        }
    }

    impl Component for And2Gates {
        type Kind = kind::Composite;
    }

    query_test!(
        name: test_and2gates_small_and_tree_dedupe_none,
        query: And2Gates,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 8,
        config: |config_builder| config_builder.dedupe(Dedupe::None)
    );

    query_test!(
        name: test_and2gates_small_and_tree_dedupe_all,
        query: And2Gates,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 2,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );
}
