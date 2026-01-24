use std::any::TypeId;

use svql_driver::{Driver, DriverKey};

use crate::{
    prelude::{ColumnDef, ColumnKind, QueryError, Table},
    session::{AnyTable, ColumnEntry, ExecutionContext, Row, Store},
    traits::{Component, PatternInternal, kind, schema_lut, search_table_any},
};

pub trait Composite: Sized + Component<Kind = kind::Composite> + Send + Sync + 'static {
    /// Schema definition for DataFrame storage.
    const SCHEMA: &'static [ColumnDef];

    /// Size of the schema (number of columns).w
    const SCHEMA_SIZE: usize = Self::SCHEMA.len();

    const CONNECTIONS: Connections;

    /// Validates a row against connectivity constraints.
    ///
    /// This handles connections between:
    /// - Two different submodules (Table A <-> Table B)
    /// - A submodule and the Composite's own IO (Table A <-> Self)
    fn validate(row: &Row<Self>, store: &Store, driver: &Driver, key: &DriverKey) -> bool {
        // 1. Access Graph
        let design_handle =
            match driver.get_design(key, &svql_common::Config::default().haystack_options) {
                Ok(d) => d,
                Err(_) => return false,
            };
        let graph = design_handle.index();

        // 2. PRE-FETCH DEPENDENCY TABLES
        // We map each column index in Self::SCHEMA to its corresponding Table in the Store.
        // If the column is a Cell (not a submodule), we store None.
        // This prevents looking up the Store inside the inner loop.
        let dep_tables: Vec<Option<&dyn AnyTable>> = Self::SCHEMA
            .iter()
            .map(|col| {
                if let ColumnKind::Sub(type_id) = col.kind {
                    store.get_any(type_id)
                } else {
                    None
                }
            })
            .collect();

        // 3. Iterate CNF Constraints
        for group in Self::CONNECTIONS.connections {
            let mut group_satisfied = false;
            let mut group_has_unresolvable = false;

            for connection in *group {
                // Pass the pre-fetched tables to resolve_endpoint
                let src_wire = connection.from.resolve_endpoint(row, &dep_tables);
                let dst_wire = connection.to.resolve_endpoint(row, &dep_tables);

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
            if !group_has_unresolvable {
                return false;
            }
        }

        true
    }

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
    const SCHEMA_SIZE: usize = T::SCHEMA_SIZE;

    const SCHEMA: &'static [ColumnDef] = T::SCHEMA;

    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: composite_search_table_any::<T>,
        nested_dependancies: &[],
    };

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        todo!();
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        todo!();
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
    ///
    /// Handles the logic of:
    /// 1. "Is this endpoint on Me (Self) or a Submodule?"
    /// 2. "If Submodule, which row in that table?"
    /// 3. "Get the specific port from that row."
    fn resolve_endpoint<T>(&self, row: &Row<T>, dep_tables: &[Option<&dyn AnyTable>]) -> Option<u64>
    where
        T: Composite + Component,
    {
        // 1. Get the value from the current Composite Row
        let entry = row.entry_array.entries.get(self.column_idx)?;

        // 2. Check the column definition to know how to handle the entry
        let col_def = &T::SCHEMA[self.column_idx];

        match (entry, &col_def.kind) {
            // === CASE 1: Endpoint is on a Submodule ===
            // The row contains a Reference (Index) to another table.
            (ColumnEntry::Sub { id: Some(ref_idx) }, ColumnKind::Sub(_)) => {
                // Retrieve the pre-fetched table for this column
                let table = dep_tables.get(self.column_idx)?.as_ref()?;

                // Ask that table for the cell ID at the specific row and port name
                table.get_cell_id(*ref_idx as usize, self.port_name)
            }

            // === CASE 2: Endpoint is on Self ===
            // The row contains the physical Cell ID directly.
            (ColumnEntry::Cell { id: Some(cell_id) }, ColumnKind::Cell) => Some(*cell_id),

            // === CASE 3: Missing Data ===
            // Nullable column or metadata
            _ => None,
        }
    }
}

pub struct Connection {
    from: Endpoint,
    to: Endpoint,
}

pub struct Connections {
    // in CNF form (helpful for some patterns like either (y -> a or y -> b) and (z -> c or z -> d))
    connections: &'static [&'static [Connection]],
}

#[allow(unused)]
mod test {

    use crate::{Wire, prelude::ColumnKind, traits::Netlist};

    use super::*;

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
        const SCHEMA: &'static [ColumnDef] = &[
            ColumnDef::new("a", ColumnKind::Cell, false),
            ColumnDef::new("b", ColumnKind::Cell, false),
            ColumnDef::new("y", ColumnKind::Cell, false),
        ];

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
        const SCHEMA: &'static [ColumnDef] = &[];

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
                        module: TypeId::of::<AndGate>(),
                        port: "b",
                    },
                    to: Endpoint {
                        module: TypeId::of::<AndGate>(),
                        port: "y",
                    },
                },
                Connection {
                    from: Endpoint {
                        module: TypeId::of::<AndGate>(),
                        port: "a",
                    },
                    to: Endpoint {
                        module: TypeId::of::<AndGate>(),
                        port: "y",
                    },
                },
            ]];
            Connections { connections: conns }
        };
    }

    impl Component for And2Gates {
        type Kind = kind::Composite;
    }

    // query_test!(
    //     name: test_and_mixed_and_tree_dedupe_none,
    //     query: AndGate,
    //     haystack: ("examples/fixtures/basic/and/json/mixed_and_tree.json", "mixed_and_tree"),
    //     expect: 6,
    //     config: |config_builder| config_builder.dedupe(Dedupe::None)
    // );

    // query_test!(
    //     name: test_and_mixed_and_tree_dedupe_all,
    //     query: AndGate,
    //     haystack: ("examples/fixtures/basic/and/json/mixed_and_tree.json", "mixed_and_tree"),
    //     expect: 3,
    //     config: |config_builder| config_builder.dedupe(Dedupe::All)
    // );
}
