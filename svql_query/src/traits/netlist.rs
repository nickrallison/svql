//! Netlist component traits and utilities.
//!
//! Provides traits for components defined via external HDL files.

use crate::prelude::*;
use svql_subgraph::{SubgraphMatcher, graph_index::IoMapping};
use tracing::debug;

/// Trait for netlist-based pattern components.
///
/// Implemented by types generated with `#[netlist]`. Provides access to
/// the source file path and module name.
pub trait Netlist {
    /// The module name within the source file.
    const MODULE_NAME: &'static str;

    /// Path to the netlist source file (.v, .il, or .json).
    const FILE_PATH: &'static str;

    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize;

    /// Schema definition for DataFrame storage.
    const SCHEMA: [ColumnDef; Self::SCHEMA_SIZE];

    /// The matched instance type.
    type RowMatch = [CellId; Self::SCHEMA_SIZE];

    /// Returns the driver key for this netlist.
    fn driver_key() -> DriverKey {
        debug!(
            "Creating driver key for netlist: {}, file: {}",
            Self::MODULE_NAME,
            Self::FILE_PATH
        );
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }

    /// Binds a subgraph assignment to produce a matched instance.
    // fn bind_match(&self, resolver: &PortResolver, assignment: &SingleAssignment) -> Self::Match;

    fn resolve(assignment: &SingleAssignment<'_, '_>, io_mapping: &IoMapping) -> Self::RowMatch {
        let mut row_match: Self::RowMatch = Default::default();
        for (name, cell_ids) in io_mapping.input_fanout_by_name_map() {
            let 
            let cell = needle_cell.get();
            match cell {
                prjunnamed_netlist::Cell::Input(name, size) => todo!(),
                prjunnamed_netlist::Cell::Output(name, value) => todo!(),
                _ => continue,
            }
        }

        todo!()
    }
}

impl<T> Pattern for T
where
    T: Netlist,
{
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
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

    fn columns() -> &'static [ColumnDef] {
        T::columns()
    }

    fn dependencies() -> &'static [std::any::TypeId] {
        &[]
    }

    fn search(ctx: &ExecutionContext<'_>) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        let needle_key = Self::driver_key();
        let haystack_key = ctx.driver_key();

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

        todo!("Finish building table from assignments");

        // // execute_netlist_query(ctx.driver(), ctx.driver_key(), ctx.config());

        //     // Early return for empty results
        //     if assignments.items.is_empty() {
        //         return ::svql_query::session::Table::empty(Self::df_columns());
        //     }

        //     let needle_container = full_context.get(&needle_key)
        //         .ok_or_else(|| QueryError::missing_dep(stringify!(#struct_name).to_string()))?;
        //     let resolver = PortResolver::new(needle_container.index());

        //     let mut builder = TableBuilder::<Self>::new(Self::df_columns());
        //     for assignment in &assignments.items {
        //         let row = Row::<Self>::new(0, search_instance.path.to_string())
        //             #(#row_wire_fields)*;
        //         builder.push(row);
        //     }

        //     builder.build()
    }

    fn rehydrate(_row: &Row<Self>, _store: &Store) -> Option<Self>
    where
        Self: 'static,
    {
        todo!()
    }
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
