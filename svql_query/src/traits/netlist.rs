//! Netlist component traits and utilities.
//!
//! Provides traits for components defined via external HDL files.

use crate::{
    prelude::*,
    session::{AnyTable, ColumnEntry, EntryArray, ExecutionContext, QueryError, Row, Store, Table},
    traits::{Component, PatternInternal, kind, schema_lut, search_table_any},
};
use prjunnamed_netlist::Design;
use svql_subgraph::SubgraphMatcher;
use tracing::debug;

/// Trait for netlist-based pattern components.
///
/// Implemented by types generated with `#[netlist]`. Provides access to
/// the source file path and module name.
pub trait Netlist: Sized + Component<Kind = kind::Netlist> + Send + Sync + 'static {
    /// The module name within the source file.
    const MODULE_NAME: &'static str;

    /// Path to the netlist source file (.v, .il, or .json).
    const FILE_PATH: &'static str;

    /// Schema definition for DataFrame storage.
    const SCHEMA: &'static [ColumnDef];

    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize = Self::SCHEMA.len();

    /// Returns the driver key for this netlist.
    fn driver_key() -> DriverKey {
        debug!(
            "Creating driver key for netlist: {}, file: {}",
            Self::MODULE_NAME,
            Self::FILE_PATH
        );
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }

    fn resolve(assignment: &SingleAssignment<'_, '_>) -> EntryArray {
        let mut row_match: Vec<Option<u64>> = vec![None; Self::SCHEMA_SIZE];
        for (haystack_cell_wrapper, needle_cell_wrapper) in assignment.haystack_mapping() {
            let needle_cell = needle_cell_wrapper.get();
            match needle_cell {
                prjunnamed_netlist::Cell::Input(name, _)
                | prjunnamed_netlist::Cell::Output(name, _) => {
                    let col_idx = schema_lut(name, Self::SCHEMA)
                        .expect("Needle Cell name should exist in schema");
                    row_match[col_idx] = Some(haystack_cell_wrapper.debug_index() as u64);
                }
                _ => continue,
            }
        }

        for idx in 0..Self::SCHEMA_SIZE {
            if row_match[idx] == None {
                let col_name = &Self::SCHEMA[idx].name;
                panic!("Unmapped column in match: {}", col_name);
            }
        }

        let final_row_match: Vec<ColumnEntry> = row_match
            .into_iter()
            .map(|opt| ColumnEntry::Cell { id: opt })
            .collect();
        EntryArray::new(final_row_match)
    }

    fn rehydrate<'a>(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static;
}

/// Wrapper to adapt the two-argument `search_table_any` to the single-argument `SearchFn` signature.
fn netlist_search_table_any<T>(
    ctx: &ExecutionContext,
) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>
where
    T: Netlist + Component<Kind = kind::Netlist> + Send + Sync + 'static,
{
    search_table_any::<T>(ctx, <T as PatternInternal<kind::Netlist>>::search_table)
}

impl<T> PatternInternal<kind::Netlist> for T
where
    T: Netlist + Component<Kind = kind::Netlist> + Send + Sync + 'static,
{
    const SCHEMA_SIZE: usize = T::SCHEMA_SIZE;

    const SCHEMA: &'static [ColumnDef] = T::SCHEMA;

    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: netlist_search_table_any::<T>,
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

        let assignments = SubgraphMatcher::enumerate_with_indices(
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

    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static,
    {
        <T as Netlist>::rehydrate(row, store, driver, key)
    }
}

mod test {

    use crate::session::CellId;

    #[allow(unused)]
    use super::*;

    use svql_query::query_test;

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

    query_test!(
        name: test_and_mixed_and_tree
        query: AndGate,
        haystack: ("examples/fixtures/basic/and/json/mixed_and_tree.json", "mixed_and_tree"),
        expect: 6,
        config: grant_access_config
    );
}

// /// Generates a report node by aggregating source information from all ports.
// pub fn report_netlist(
//     path: &Instance,
//     type_name: &'static str,
//     wires: &[&Wire<Match>],
// ) -> ReportNode {
//     let mut all_lines = Vec::new();
//     let mut file_path = std::sync::Arc::from("");
//     let mut seen = std::collections::HashSet::new();

//     for wire in wires {
//         if let Some(loc) = wire.inner.as_ref().and_then(|c| c.get_source()) {
//             file_path = loc.file;
//             for line in loc.lines {
//                 if seen.insert(line.number) {
//                     all_lines.push(line);
//                 }
//             }
//         }
//     }

//     all_lines.sort_by_key(|l| l.number);

//     ReportNode {
//         name: String::new(),
//         type_name: type_name.to_string(),
//         path: path.clone(),
//         details: None,
//         source_loc: if file_path.is_empty() {
//             None
//         } else {
//             Some(SourceLocation {
//                 file: file_path,
//                 lines: all_lines,
//             })
//         },
//         children: Vec::new(),
//     }
// }
