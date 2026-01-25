use itertools::Itertools;
use svql_driver::{Driver, DriverKey};

use crate::{
    prelude::{ColumnDef, ColumnKind, QueryError, Table},
    session::{AnyTable, ColumnEntry, EntryArray, ExecInfo, ExecutionContext, Row, Store},
    traits::{Component, PatternInternal, kind, schema_lut, search_table_any},
};

pub trait Composite: Sized + Component<Kind = kind::Composite> + Send + Sync + 'static {
    /// Schema definition for DataFrame storage.
    const DEFS: &'static [ColumnDef];

    /// Size of the schema (number of columns).w
    const SCHEMA_SIZE: usize = Self::DEFS.len();

    const CONNECTIONS: Connections;

    const DEPENDANCIES: &'static [&'static ExecInfo];

    /// Access the smart Schema wrapper.
    fn schema() -> &'static crate::session::PatternSchema;

    /// Core logic to compose sub-matches into a result table.
    ///
    /// # Arguments
    /// * `ctx` - The execution context (access to driver, config, etc).
    /// * `dep_tables` - A slice aligned 1:1 with `Self::DEFS`.
    ///   - If `DEFS[i]` is a `Sub`, `dep_tables[i]` contains `Some(table)`.
    ///   - If `DEFS[i]` is a `Cell` (Wire), `dep_tables[i]` is `None`.
    fn compose(
        ctx: &ExecutionContext,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
    ) -> Result<Table<Self>, QueryError> {
        // 1. Use pre-computed submodule indices from the smart Schema
        let schema = Self::schema();
        let sub_indices = &schema.submodules;

        for idx in sub_indices.iter() {
            println!("Submodule column at index: {}", idx);
        }

        // 2. Prepare Iterators (Same logic, but using sub_indices)
        let mut ranges = Vec::with_capacity(sub_indices.len());

        for (i, &col_idx) in sub_indices.iter().enumerate() {
            let table = dep_tables[i];

            if table.is_empty() {
                // If a required submodule has no matches, the composite has no matches.
                // (Assumes non-nullable for now)
                if !schema.column(col_idx).nullable {
                    return Ok(Table::new(vec![])?);
                }
            }
            ranges.push(0..table.len() as u64);
        }

        // 3. Perform Cartesian Product
        // This creates an iterator of Vec<u64> (row indices), one for each submodule.
        let product_iter = ranges.into_iter().multi_cartesian_product();

        // 4. Build and Validate Rows
        let mut valid_rows = Vec::new();

        // Pre-allocate a template row with Metadata/None to avoid re-allocating per iteration
        let row_template = vec![ColumnEntry::Metadata { id: None }; Self::SCHEMA_SIZE];

        for indices in product_iter {
            let mut entries = row_template.clone();

            // Map the Cartesian product indices back to their specific column positions
            for (i, &row_idx) in indices.iter().enumerate() {
                let schema_col_idx = sub_indices[i];
                entries[schema_col_idx] = ColumnEntry::Sub { id: Some(row_idx) };
            }

            let row = Row::<Self> {
                idx: 0, // Placeholder
                entry_array: EntryArray::new(entries),
                _marker: std::marker::PhantomData,
            };

            // 5. Validate
            if Self::validate(&row, dep_tables, ctx.driver(), &ctx.design_key()) {
                valid_rows.push(row.entry_array);
            }
        }

        Table::new(valid_rows)
    }

    /// Validates a row against connectivity constraints.
    ///
    /// Refactored to accept `dep_tables` directly, allowing validation during
    /// the `compose` phase before the final Table is fully built.
    fn validate(
        row: &Row<Self>,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
        driver: &Driver,
        key: &DriverKey,
    ) -> bool {
        // 1. Access Graph
        // Note: In a hot loop, you might want to hoist this graph lookup out of validate
        // and pass the `GraphIndex` directly, but this signature matches the trait.
        let design_handle =
            match driver.get_design(key, &svql_common::Config::default().haystack_options) {
                Ok(d) => d,
                Err(_) => return false,
            };
        let graph = design_handle.index();

        // 2. Iterate CNF Constraints
        for group in Self::CONNECTIONS.connections {
            let mut group_satisfied = false;
            let mut group_has_unresolvable = false;

            for connection in *group {
                // Pass the pre-fetched tables to resolve_endpoint
                let src_wire = connection.from.resolve_endpoint(row, dep_tables);
                let dst_wire = connection.to.resolve_endpoint(row, dep_tables);

                match (src_wire, dst_wire) {
                    (Some(s), Some(d)) => {
                        // Check physical connectivity in the netlist graph
                        if graph.is_connected(s, d) {
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

/// Wrapper to adapt the two-argument `search_table_any` to the single-argument `SearchFn` signature.
fn composite_search_table_any<T>(
    ctx: &ExecutionContext,
) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>
where
    T: Composite + Component<Kind = kind::Composite> + Send + Sync + 'static,
{
    search_table_any::<T>(ctx, <T as PatternInternal<kind::Composite>>::search_table)
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
        search_function: composite_search_table_any::<T>,
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
    /// The index of the column in Self::SCHEMA.
    /// - If SCHEMA[idx] is a `Sub`, this refers to a port on that submodule.
    /// - If SCHEMA[idx] is a `Cell` (Input/Output), this refers to the composite's own wire.
    pub column_idx: usize,
    /// The name of the port.
    /// - If target is a Submodule, this is the port name on that submodule (e.g., "y").
    /// - If target is Self (Cell), this is ignored (or should match the column name).
    pub port_name: &'static str,
}

impl Endpoint {
    /// Resolves an endpoint to a physical Cell ID.
    fn resolve_endpoint<T>(
        &self,
        row: &Row<T>,
        // This signature must match exactly what is passed in validate
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
    ) -> Option<u64>
    where
        T: Composite + Component,
    {
        // 1. Get the value from the current Composite Row
        let entry = row.entry_array.entries.get(self.column_idx)?;

        // 2. Check the column definition
        let col_def = T::schema().column(self.column_idx);

        match (entry, &col_def.kind) {
            // === CASE 1: Endpoint is on a Submodule ===
            (ColumnEntry::Sub { id: Some(ref_idx) }, ColumnKind::Sub(_)) => {
                // Retrieve the pre-fetched table for this column
                let sub_idx = T::schema()
                    .submodules
                    .iter()
                    .position(|&idx| idx == self.column_idx)?;
                let table = dep_tables.get(sub_idx)?;

                // Ask that table for the cell ID at the specific row and port name
                table.get_cell_id(*ref_idx as usize, self.port_name)
            }

            // === CASE 2: Endpoint is on Self ===
            (ColumnEntry::Cell { id: Some(cell_id) }, ColumnKind::Cell) => Some(*cell_id),

            // === CASE 3: Missing Data ===
            _ => None,
        }
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

        and1_a: Wire,
        and1_b: Wire,
        and1_y: Wire,

        and2_a: Wire,
        and2_b: Wire,
        and2_y: Wire,
    }

    impl Composite for And2Gates {
        const DEFS: &'static [ColumnDef] = &[
            // 0: Submodule and1
            ColumnDef::sub::<AndGate>("and1"),
            // 1: Submodule and2
            ColumnDef::sub::<AndGate>("and2"),
            // 2-7: Wires (Exposed ports for the composite)
            ColumnDef::wire("and1_a"),
            ColumnDef::wire("and1_b"),
            ColumnDef::wire("and1_y"),
            ColumnDef::wire("and2_a"),
            ColumnDef::wire("and2_b"),
            ColumnDef::wire("and2_y"),
        ];

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
                Connection {
                    from: Endpoint {
                        column_idx: schema_lut("b", <AndGate as Netlist>::DEFS)
                            .expect("Should have successfully looked up col"),
                        port_name: "b",
                    },
                    to: Endpoint {
                        column_idx: schema_lut("y", <AndGate as Netlist>::DEFS)
                            .expect("Should have successfully looked up col"),
                        port_name: "y",
                    },
                },
                Connection {
                    from: Endpoint {
                        column_idx: schema_lut("a", <AndGate as Netlist>::DEFS)
                            .expect("Should have successfully looked up col"),
                        port_name: "a",
                    },
                    to: Endpoint {
                        column_idx: schema_lut("y", <AndGate as Netlist>::DEFS)
                            .expect("Should have successfully looked up col"),
                        port_name: "y",
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
        expect: 6,
        config: |config_builder| config_builder.dedupe(Dedupe::None)
    );

    query_test!(
        name: test_and2gates_small_and_tree_dedupe_all,
        query: And2Gates,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );
}
