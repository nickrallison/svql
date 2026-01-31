//! Netlist component traits and utilities.
//!
//! Provides traits for components defined via external HDL files.

use crate::prelude::*;
use prjunnamed_netlist::Value;
use tracing::debug;

fn value_to_cell_id(value: &Value) -> Option<u32> {
    match value.as_net() {
        Some(net) => net.as_cell_index().map(|idx| idx as u32).ok(),
        None => None,
    }
}

/// Trait for netlist-based pattern components.
///
/// Implemented by types generated with `#[netlist]`. Provides access to
/// the source file path and module name.
pub trait Netlist: Sized + Component<Kind = kind::Netlist> + Send + Sync + 'static {
    /// The module name within the source file.
    const MODULE_NAME: &'static str;

    /// Path to the netlist source file (.v, .il, or .json).
    const FILE_PATH: &'static str;

    /// Port declarations (macro-generated)
    const PORTS: &'static [Port];

    /// Schema accessor (macro generates this with `OnceLock` pattern)
    fn netlist_schema() -> &'static crate::session::PatternSchema {
        static SCHEMA: std::sync::OnceLock<crate::session::PatternSchema> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::ports_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            crate::session::PatternSchema::new(defs_static)
        })
    }

    /// Convert port declarations to column definitions
    #[must_use] 
    fn ports_to_defs() -> Vec<ColumnDef> {
        Self::PORTS
            .iter()
            .map(|p| ColumnDef::new(p.name, ColumnKind::Cell, false).with_direction(p.direction))
            .collect()
    }

    /// Returns the driver key for this netlist.
    fn driver_key() -> DriverKey {
        debug!(
            "Creating driver key for netlist: {}, file: {}",
            Self::MODULE_NAME,
            Self::FILE_PATH
        );
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }

    #[must_use] 
    fn resolve(assignment: &SingleAssignment<'_, '_>) -> EntryArray {
        let schema_size = Self::PORTS.len();
        let mut row_match: Vec<Option<u32>> = vec![None; schema_size];
        for (haystack_cell_wrapper, needle_cell_wrapper) in assignment.haystack_mapping() {
            let needle_cell = needle_cell_wrapper.get();
            match needle_cell {
                prjunnamed_netlist::Cell::Input(name, _) => {
                    let col_idx = Self::netlist_schema()
                        .index_of(name)
                        .expect("Needle Cell name should exist in schema");
                    row_match[col_idx] = Some(haystack_cell_wrapper.debug_index() as u32);
                }
                prjunnamed_netlist::Cell::Output(name, output_value) => {
                    let col_idx = Self::netlist_schema()
                        .index_of(name)
                        .expect("Needle Cell name should exist in schema");
                    let needle_output_driver_id: u32 =
                        value_to_cell_id(output_value).expect("Output should have driver");
                    let haystack_output_driver_wrapper = assignment
                        .needle_mapping()
                        .iter()
                        .find(|(needle_cell_wrapper, _haystack_cell_wrapper)| {
                            needle_cell_wrapper.debug_index() as u32 == needle_output_driver_id
                        })
                        .map(|(_needle_cell_wrapper, haystack_cell_wrapper)| haystack_cell_wrapper)
                        .expect("Should find haystack driver for output");

                    row_match[col_idx] = Some(haystack_output_driver_wrapper.debug_index() as u32);
                }
                _ => continue,
            }
        }

        for (idx, item) in row_match.iter().enumerate().take(schema_size) {
            if item.is_none() {
                let col_name = Self::netlist_schema().column(idx).name;
                panic!("Unmapped column in match: {col_name}");
            }
        }

        let final_row_match: Vec<ColumnEntry> = row_match
            .into_iter()
            .map(|opt| ColumnEntry::Cell {
                id: opt.map(CellId::new),
            })
            .collect();
        EntryArray::new(final_row_match)
    }

    fn netlist_rehydrate(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static;
}

impl<T> PatternInternal<kind::Netlist> for T
where
    T: Netlist + Component<Kind = kind::Netlist> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = &[]; // Placeholder, not used anymore

    const SCHEMA_SIZE: usize = T::PORTS.len();

    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: |ctx| {
            search_table_any::<T>(ctx, <T as PatternInternal<kind::Netlist>>::search_table)
        },
        nested_dependancies: &[],
    };

    fn internal_schema() -> &'static crate::session::PatternSchema {
        T::netlist_schema()
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        driver.preload_design(&Self::driver_key(), &config.needle_options)?;
        driver.preload_design(design_key, &config.haystack_options)?;
        Ok(())
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        let needle_key = Self::driver_key();
        let haystack_key = ctx.design_key();

        let needle_container = ctx
            .driver()
            .get_design(&needle_key, &ctx.config().needle_options)
            .map_err(|e| QueryError::needle_load(e.to_string()))?;
        let haystack_container = ctx
            .driver()
            .get_design(&haystack_key, &ctx.config().haystack_options)
            .map_err(|e| QueryError::design_load(e.to_string()))?;

        let assignments = subgraph::SubgraphMatcher::enumerate_with_indices(
            needle_container.design(),
            haystack_container.design(),
            needle_container.index(),
            haystack_container.index(),
            needle_key.module_name().to_string(),
            haystack_key.module_name().to_string(),
            ctx.config(),
        );

        // todo!();

        let row_matches: Vec<EntryArray> = assignments
            .items
            .iter()
            .map(|assignment| Self::resolve(assignment))
            .collect();

        let table = Table::<Self>::new(row_matches)?;
        Ok(table)
    }

    fn internal_rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static,
    {
        Self::netlist_rehydrate(row, store, driver, key)
    }
}

#[allow(unused)]
pub(crate) mod test {

    use crate::Wire;

    use super::{Netlist, Component, kind, Port, Row, Store, Driver, DriverKey};
    use svql_common::Dedupe;

    use svql_query::query_test;

    #[derive(Debug, Clone, Netlist)]
    #[netlist(
        file = "examples/fixtures/basic/and/verilog/and_gate.v",
        module = "and_gate"
    )]
    pub struct AndGate {
        #[port(input)]
        pub a: Wire,
        #[port(input)]
        pub b: Wire,
        #[port(output)]
        pub y: Wire,
    }

    query_test!(
        name: test_and_mixed_and_tree_dedupe_none,
        query: AndGate,
        haystack: ("examples/fixtures/basic/and/json/mixed_and_tree.json", "mixed_and_tree"),
        expect: 6,
        config: |config_builder| config_builder.dedupe(Dedupe::None)
    );

    query_test!(
        name: test_and_mixed_and_tree_dedupe_all,
        query: AndGate,
        haystack: ("examples/fixtures/basic/and/json/mixed_and_tree.json", "mixed_and_tree"),
        expect: 3,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );

    // --- Reference Implementation (Manual) ---

    #[derive(Debug, Clone)]
    pub struct ManualAndGate {
        pub a: Wire,
        pub b: Wire,
        pub y: Wire,
    }

    impl Component for ManualAndGate {
        type Kind = kind::Netlist;
    }

    impl Netlist for ManualAndGate {
        const MODULE_NAME: &'static str = "and_gate";
        const FILE_PATH: &'static str = "examples/fixtures/basic/and/verilog/and_gate.v";

        const PORTS: &'static [Port] = &[Port::input("a"), Port::input("b"), Port::output("y")];

        fn netlist_rehydrate(
            row: &Row<Self>,
            _store: &Store,
            _driver: &Driver,
            _key: &DriverKey,
        ) -> Option<Self> {
            Some(Self {
                a: row.wire("a")?,
                b: row.wire("b")?,
                y: row.wire("y")?,
            })
        }
    }

    query_test!(
        name: test_manual_and_gate_small_tree,
        query: ManualAndGate,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3
    );
}
