use svql_driver::{Driver, DriverKey};

use crate::{
    prelude::{ColumnDef, QueryError, Table},
    session::{AnyTable, ColumnEntry, EntryArray, ExecInfo, ExecutionContext, Row, Store},
    traits::{Component, PatternInternal, kind, search_table_any},
};

/// Maps a common port name to an inner type's port name.
///
/// Used by Variant types to unify different port names from inner types
/// into a single common interface.
#[derive(Debug, Clone, Copy)]
pub struct PortMapping {
    /// The port name exposed by the variant (unified interface).
    pub common_port: &'static str,
    /// The port name in the inner type that maps to this common port.
    pub inner_port: &'static str,
}

/// Describes one variant arm for lookup during rehydration.
///
/// Contains metadata about each variant arm to enable runtime dispatch
/// during rehydration.
#[derive(Debug, Clone, Copy)]
pub struct VariantArm {
    /// The TypeId of the inner pattern type.
    pub type_id: std::any::TypeId,
    /// Human-readable type name for debugging.
    pub type_name: &'static str,
}

pub trait Variant: Sized + Component<Kind = kind::Variant> + Send + Sync + 'static {
    /// Schema definition for DataFrame storage.
    ///
    /// Must include:
    /// - `discriminant` (Metadata): which variant arm matched (0, 1, 2, ...)
    /// - `inner_ref` (Sub<Self>): row index into variant arm's table
    /// - Common ports (Cell): mapped from inner types
    const DEFS: &'static [ColumnDef];

    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize = Self::DEFS.len();

    /// Number of variant arms.
    const NUM_VARIANTS: usize;

    /// Port mappings for each variant arm.
    ///
    /// Index corresponds to discriminant value.
    /// `PORT_MAPS[variant_idx]` gives the mappings for that arm.
    const PORT_MAPS: &'static [&'static [PortMapping]];

    /// Variant arm metadata (TypeId + name for each arm).
    const VARIANT_ARMS: &'static [VariantArm];

    /// Dependencies: ExecInfo for each variant arm (in discriminant order).
    const DEPENDANCIES: &'static [&'static ExecInfo];

    /// Access the smart Schema wrapper.
    fn schema() -> &'static crate::session::PatternSchema;

    /// Concatenate results from all variant arms into a unified table.
    ///
    /// This is the core operation for variants - it unions the results from
    /// each inner type, mapping their ports to the common interface and
    /// tracking which arm each row came from via the discriminant.
    fn concatenate(
        _ctx: &ExecutionContext,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
    ) -> Result<Table<Self>, QueryError> {
        let schema = Self::schema();
        let mut all_entries = Vec::new();

        // Column indices (cached once)
        let discrim_idx = schema
            .index_of("discriminant")
            .ok_or_else(|| QueryError::SchemaLut("discriminant".to_string()))?;
        let inner_ref_idx = schema
            .index_of("inner_ref")
            .ok_or_else(|| QueryError::SchemaLut("inner_ref".to_string()))?;

        for (variant_idx, table) in dep_tables.iter().enumerate() {
            let port_map = Self::PORT_MAPS[variant_idx];

            for row_idx in 0..table.len() as u64 {
                let mut entry = EntryArray::with_capacity(Self::SCHEMA_SIZE);

                // Initialize all entries to None (already done by with_capacity for Metadata)
                // Re-initialize cells explicitly
                for i in 0..Self::SCHEMA_SIZE {
                    entry.entries[i] = ColumnEntry::Cell { id: None };
                }

                // 1. Set discriminant
                entry.entries[discrim_idx] = ColumnEntry::Metadata {
                    id: Some(variant_idx as u64),
                };

                // 2. Set inner_ref (row index in the variant arm's table)
                entry.entries[inner_ref_idx] = ColumnEntry::Sub { id: Some(row_idx) };

                // 3. Map common ports from inner table
                for mapping in port_map.iter() {
                    if let Some(col_idx) = schema.index_of(mapping.common_port) {
                        let cell_id = table.get_cell_id(row_idx as usize, mapping.inner_port);
                        entry.entries[col_idx] = ColumnEntry::Cell { id: cell_id };
                    }
                }

                all_entries.push(entry);
            }
        }

        Table::new(all_entries)
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized;

    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Variant> + Send + Sync + 'static;
}

impl<T> PatternInternal<kind::Variant> for T
where
    T: Variant + Component<Kind = kind::Variant> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = T::DEFS;

    const SCHEMA_SIZE: usize = T::SCHEMA_SIZE;

    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: |ctx| {
            search_table_any::<T>(ctx, <T as PatternInternal<kind::Variant>>::search_table)
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
        <T as Variant>::preload_driver(driver, design_key, config)
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        // 1. Gather dependency tables (one per variant arm)
        let mut dep_tables: Vec<&(dyn AnyTable + Send + Sync)> =
            Vec::with_capacity(T::NUM_VARIANTS);

        for (i, dep_info) in T::DEPENDANCIES.iter().enumerate() {
            let table = ctx.get_any_table(dep_info.type_id).ok_or_else(|| {
                QueryError::MissingDependency(format!(
                    "Variant arm {}: {} (TypeId {:?})",
                    i, dep_info.type_name, dep_info.type_id
                ))
            })?;
            dep_tables.push(table);
        }

        // 2. Concatenate results from all arms
        T::concatenate(ctx, &dep_tables)
    }

    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Variant> + Send + Sync + 'static,
    {
        <T as Variant>::rehydrate(row, store, driver, key)
    }
}

#[allow(unused)]
mod test {

    use crate::{
        Wire,
        prelude::ColumnKind,
        selector::Selector,
        session::ExecInfo,
        traits::{
            Netlist, Pattern,
            composite::{Composite, Connection, Connections, Endpoint},
            schema_lut,
        },
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

    #[derive(Debug, Clone)]
    enum AndOrAnd2 {
        AndGate(AndGate),
        And2Gates(And2Gates),
    }

    impl Variant for AndOrAnd2 {
        const DEFS: &'static [ColumnDef] = &[
            ColumnDef::metadata("discriminant"),
            ColumnDef::sub::<Self>("inner_ref"),
            ColumnDef::input("a"),
            ColumnDef::input("b"),
            ColumnDef::output("y"),
        ];

        const NUM_VARIANTS: usize = 2;

        const PORT_MAPS: &'static [&'static [PortMapping]] = &[
            // Variant 0: AndGate
            &[
                PortMapping {
                    common_port: "a",
                    inner_port: "a",
                },
                PortMapping {
                    common_port: "b",
                    inner_port: "b",
                },
                PortMapping {
                    common_port: "y",
                    inner_port: "y",
                },
            ],
            // Variant 1: And2Gates - maps to first and gate's inputs and second's output
            &[
                PortMapping {
                    common_port: "a",
                    inner_port: "and1.a",
                },
                PortMapping {
                    common_port: "b",
                    inner_port: "and1.b",
                },
                PortMapping {
                    common_port: "y",
                    inner_port: "and2.y",
                },
            ],
        ];

        const VARIANT_ARMS: &'static [VariantArm] = &[
            VariantArm {
                type_id: std::any::TypeId::of::<AndGate>(),
                type_name: "AndGate",
            },
            VariantArm {
                type_id: std::any::TypeId::of::<And2Gates>(),
                type_name: "And2Gates",
            },
        ];

        const DEPENDANCIES: &'static [&'static ExecInfo] = &[
            <AndGate as Pattern>::EXEC_INFO,
            <And2Gates as Pattern>::EXEC_INFO,
        ];

        fn schema() -> &'static crate::session::PatternSchema {
            static INSTANCE: std::sync::OnceLock<crate::session::PatternSchema> =
                std::sync::OnceLock::new();
            INSTANCE.get_or_init(|| crate::session::PatternSchema::new(<Self as Variant>::DEFS))
        }

        fn rehydrate<'a>(
            row: &Row<Self>,
            store: &Store,
            driver: &Driver,
            key: &DriverKey,
        ) -> Option<Self>
        where
            Self: Component + PatternInternal<kind::Variant> + Send + Sync + 'static,
        {
            // 1. Get discriminant
            let discrim_idx = <Self as Variant>::schema().index_of("discriminant")?;
            let discrim = row.entry_array.entries.get(discrim_idx)?.as_u64()?;

            // 2. Get inner_ref
            let inner_ref_idx = <Self as Variant>::schema().index_of("inner_ref")?;
            let inner_row_idx = row.entry_array.entries.get(inner_ref_idx)?.as_u64()?;

            // 3. Dispatch based on discriminant
            match discrim {
                0 => {
                    // AndGate
                    let inner_table = store.get::<AndGate>()?;
                    let inner_row = inner_table.row(inner_row_idx)?;
                    let inner = <AndGate as Pattern>::rehydrate(&inner_row, store, driver, key)?;
                    Some(AndOrAnd2::AndGate(inner))
                }
                1 => {
                    // And2Gates
                    let inner_table = store.get::<And2Gates>()?;
                    let inner_row = inner_table.row(inner_row_idx)?;
                    let inner = <And2Gates as Pattern>::rehydrate(&inner_row, store, driver, key)?;
                    Some(AndOrAnd2::And2Gates(inner))
                }
                _ => None,
            }
        }

        fn preload_driver(
            driver: &Driver,
            design_key: &DriverKey,
            config: &svql_common::Config,
        ) -> Result<(), Box<dyn std::error::Error>>
        where
            Self: Sized,
        {
            <AndGate as Pattern>::preload_driver(driver, design_key, config)?;
            <And2Gates as Pattern>::preload_driver(driver, design_key, config)?;
            Ok(())
        }
    }

    impl Component for AndOrAnd2 {
        type Kind = kind::Variant;
    }

    query_test!(
        name: test_and_mixed_and_tree_dedupe_none,
        query: AndOrAnd2,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 6,
        config: |config_builder| config_builder.dedupe(Dedupe::None)
    );

    query_test!(
        name: test_and_mixed_and_tree_dedupe_all,
        query: AndOrAnd2,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 0,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );
}
