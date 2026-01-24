use svql_driver::{Driver, DriverKey};

use crate::{
    prelude::{ColumnDef, QueryError, Table},
    session::{AnyTable, ExecutionContext, Row, Store},
    traits::{Component, PatternInternal, kind, search_table_any},
};

pub trait Composite: Sized + Component<Kind = kind::Composite> + Send + Sync + 'static {
    /// Schema definition for DataFrame storage.
    const SCHEMA: &'static [ColumnDef];

    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize = Self::SCHEMA.len();

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

#[allow(unused)]
mod test {

    use crate::{prelude::ColumnKind, session::CellId, traits::Netlist};

    use super::*;

    use svql_query::query_test;

    #[derive(Debug, Clone)]
    struct AndGate {
        a: CellId,
        b: CellId,
        y: CellId,
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
            let a_id = CellId::new(row.wire("a")?);
            let b_id = CellId::new(row.wire("b")?);
            let y_id = CellId::new(row.wire("y")?);

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
