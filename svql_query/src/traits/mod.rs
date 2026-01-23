//! Core traits for query components and execution.
//!
//! Defines the interfaces that all netlists, composites, and variants must
//! implement to participate in the SVQL query system.

use std::any::TypeId;

// pub mod component;
// pub mod composite;
pub mod netlist;
// pub mod variant;

// Re-export key traits
// pub use component::{MatchedComponent, SearchableComponent, kind};
// pub use composite::{
//     CompositeComponent, CompositeMatched, ConnectionBuilder, Connections, Topology,
//     validate_composite,
// };
pub use netlist::Netlist;
// pub use variant::{VariantComponent, VariantMatched};

use crate::prelude::*;
use crate::session::{
    AnyTable, ColumnDef, ExecInfo, ExecutionContext, ExecutionPlan, QueryError, Row, Store, Table,
};

/// The central abstraction for query components.
pub trait Pattern: Sized + Send + Sync {
    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize;

    /// Schema definition for DataFrame storage.
    const SCHEMA: &'static [ColumnDef];

    /// Info needed to execute this pattern. Used to build the ExecutionPlan DAG.
    const EXEC_INFO: &'static ExecInfo;

    /// Returns the static type name of the component.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Returns the column index for a given column name in the schema.
    fn schema_lut(name: &str) -> Option<usize> {
        for (idx, col) in Self::SCHEMA.iter().enumerate() {
            if col.name == name {
                return Some(idx);
            }
        }
        None
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized;

    /// Execute the search and return results as a Table.
    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static;

    /// Execute the search and return results as a boxed AnyTable.
    fn search_table_any(ctx: &ExecutionContext) -> Result<Box<dyn AnyTable>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        let table = Self::search_table(ctx)?;
        Ok(Box::new(table))
    }

    fn search(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<Store, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        let plan = ExecutionPlan::build(Self::EXEC_INFO);
        plan.execute(&driver, &design_key, &config)
    }

    /// Rehydrate a Row back to the Match type-state.
    ///
    /// Reconstructs a full `Self::Match` object from the row data and
    /// any referenced submodule rows in the store.
    fn rehydrate(_row: &Row<Self>, _store: &Store) -> Option<Self>
    where
        Self: 'static;
}

// /// Validates that a physical connection exists between two matched wires in the haystack.
// pub fn validate_connection<'ctx>(
//     from: &CellId,
//     to: &CellId,
//     haystack_index: &GraphIndex<'ctx>,
// ) -> bool {
//     // validate_connection_inner(from, to, haystack_index).unwrap_or(false)
//     todo!();
// }

// /// Internal helper to resolve CellInfo to CellWrappers and check connectivity.
// fn validate_connection_inner<'ctx>(
//     from: &CellId,
//     to: &CellId,
//     haystack_index: &GraphIndex<'ctx>,
// ) -> Option<bool> {
//     let from_id = from.cell_idx();
//     let to_id = to.cell_idx();

//     let f_wrapper = haystack_index.get_cell_by_id(from_id as usize)?;
//     let t_wrapper = haystack_index.get_cell_by_id(to_id as usize)?;

//     let fanout = haystack_index.fanout_set(f_wrapper)?;
//     Some(fanout.contains(t_wrapper))
// }
