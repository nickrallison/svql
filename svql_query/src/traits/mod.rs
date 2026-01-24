//! Core traits for query components and execution.
//!
//! Defines the interfaces that all netlists, composites, and variants must
//! implement to participate in the SVQL query system.

pub mod composite;
pub mod netlist;
pub mod variant;

// Re-export key traits
pub use netlist::Netlist;

use crate::prelude::*;
use crate::session::{
    AnyTable, ColumnDef, ExecInfo, ExecutionContext, ExecutionPlan, PortDirection, QueryError, Row,
    Store, Table,
};

/// Returns the column index for a given column name in the schema.
pub fn schema_lut(name: &str, schema: &[ColumnDef]) -> Option<usize> {
    for (idx, col) in schema.iter().enumerate() {
        if col.name == name {
            return Some(idx);
        }
    }
    None
}

pub fn search_table_any<T>(
    ctx: &ExecutionContext,
    search_table: fn(&ExecutionContext) -> Result<Table<T>, QueryError>,
) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>
where
    T: Send + Sync + Pattern + Component + 'static,
{
    let table = search_table(ctx)?;
    Ok(Box::new(table))
}

/// The central abstraction for query components.
pub trait Pattern: Sized + Send + Sync {
    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize;

    /// Schema definition for DataFrame storage.
    const SCHEMA: &'static [ColumnDef];

    /// Info needed to execute this pattern. Used to build the ExecutionPlan DAG.
    const EXEC_INFO: &'static ExecInfo;

    /// Get the indices of all Input columns in the Schema.
    fn input_indices() -> Vec<usize> {
        Self::SCHEMA
            .iter()
            .enumerate()
            .filter(|(_, col)| col.direction == PortDirection::Input)
            .map(|(i, _)| i)
            .collect()
    }

    /// Get the indices of all Output columns in the Schema.
    fn output_indices() -> Vec<usize> {
        Self::SCHEMA
            .iter()
            .enumerate()
            .filter(|(_, col)| col.direction == PortDirection::Output)
            .map(|(i, _)| i)
            .collect()
    }

    /// Check if a specific column name is an Output.
    fn is_output(name: &str) -> bool {
        Self::SCHEMA
            .iter()
            .any(|col| col.name == name && col.direction == PortDirection::Output)
    }

    /// Check if a specific column name is an Input.
    fn is_input(name: &str) -> bool {
        Self::SCHEMA
            .iter()
            .any(|col| col.name == name && col.direction == PortDirection::Input)
    }

    /// Returns the static type name of the component.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
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
        Self: Send + Sync + Component + 'static;

    /// Execute the search and return results as a boxed AnyTable.
    fn search_table_any(
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>
    where
        Self: Send + Sync + Component + 'static,
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
    fn rehydrate(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + Send + Sync + 'static;
}

pub mod kind {
    pub struct Netlist;
    pub struct Composite;
    pub struct Variant;
}

pub trait Component {
    type Kind;
}

impl Component for kind::Netlist {
    type Kind = kind::Netlist;
}

impl Component for kind::Composite {
    type Kind = kind::Composite;
}

impl Component for kind::Variant {
    type Kind = kind::Variant;
}

pub trait PatternInternal<K>: Sized {
    const SCHEMA_SIZE: usize;

    const SCHEMA: &'static [ColumnDef];

    const EXEC_INFO: &'static crate::session::ExecInfo;

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static;

    fn rehydrate(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<Self::Kind> + Send + Sync + 'static;
}

impl<T> Pattern for T
where
    T: Component,                // It has a Kind
    T: PatternInternal<T::Kind>, // It implements the logic for that Kind
    T: Send + Sync + 'static,
{
    const SCHEMA_SIZE: usize = T::SCHEMA_SIZE;

    const SCHEMA: &'static [ColumnDef] = T::SCHEMA;

    const EXEC_INFO: &'static crate::session::ExecInfo = T::EXEC_INFO;

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        T::preload_driver(driver, design_key, config)
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        T::search_table(ctx)
    }

    fn rehydrate(row: &Row<Self>, store: &Store, driver: &Driver, key: &DriverKey) -> Option<Self>
    where
        Self: 'static,
    {
        T::rehydrate(row, store, driver, key)
    }
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
