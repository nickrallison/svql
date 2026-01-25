use svql_driver::{Driver, DriverKey};

use crate::{
    prelude::{ColumnDef, QueryError, Table},
    session::{AnyTable, ExecutionContext, Row, Store},
    traits::{Component, PatternInternal, kind, search_table_any},
};

pub trait Variant: Sized + Component<Kind = kind::Variant> + Send + Sync + 'static {
    /// Schema definition for DataFrame storage.
    const DEFS: &'static [ColumnDef];

    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize = Self::DEFS.len();

    /// Access the smart Schema wrapper.
    fn schema() -> &'static crate::session::PatternSchema;

    // the rest is tbd
    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Variant> + Send + Sync + 'static;

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized;
}

/// Wrapper to adapt the two-argument `search_table_any` to the single-argument `SearchFn` signature.
fn variant_search_table_any<T>(
    ctx: &ExecutionContext,
) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>
where
    T: Variant + Component<Kind = kind::Variant> + Send + Sync + 'static,
{
    search_table_any::<T>(ctx, <T as PatternInternal<kind::Variant>>::search_table)
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
        search_function: variant_search_table_any::<T>,
        nested_dependancies: &[],
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

    fn search_table(_ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
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
        session::ExecInfo,
        traits::{
            Netlist, Pattern,
            composite::{Composite, Connection, Connections, Endpoint},
            schema_lut,
        },
    };

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

    enum AndOrAnd2 {
        AndGate(AndGate),
        And2Gates(And2Gates),
    }

    impl Variant for AndOrAnd2 {
        const DEFS: &'static [ColumnDef] = &[];

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
            todo!()
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
