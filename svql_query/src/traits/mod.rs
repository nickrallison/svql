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
use crate::session::{ColumnDef, QueryError, Table};

/// The central abstraction for query components.
pub trait Pattern: Sized {
    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize;

    /// Schema definition for DataFrame storage.
    const SCHEMA: &'static [ColumnDef];

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

    /// Dependencies as TypeIds.
    ///
    /// Returns the TypeIds of all submodule patterns that must be searched
    /// before this pattern. Generated from `#[submodule]` fields.
    fn dependencies() -> &'static [TypeId];

    /// Register this pattern and all dependencies into the registry.
    ///
    /// Called during `ExecutionPlan::for_pattern::<P>()` to build the DAG.
    /// Implementations should:
    /// 1. Call `register_all()` on each dependency
    /// 2. Call `registry.register()` for self
    fn register_all(registry: &mut PatternRegistry)
    where
        Self: 'static,
    {
        registry.register(
            TypeId::of::<Self>(),
            std::any::type_name::<Self>(),
            Self::dependencies(),
        );
    }

    /// Execute the search and return results as a Table.
    ///
    /// This is the new DataFrame-based search API. The `ExecutionContext`
    /// provides access to:
    /// - `ctx.driver()` - For design/needle operations
    /// - `ctx.driver_key()` - The design being searched
    /// - `ctx.get::<Dep>()` - Tables of completed dependencies
    fn search(_ctx: &ExecutionContext<'_>) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static;

    /// Rehydrate a Row back to the Match type-state.
    ///
    /// Reconstructs a full `Self::Match` object from the row data and
    /// any referenced submodule rows in the store.
    fn rehydrate(_row: &Row<Self>, _store: &Store) -> Option<Self>
    where
        Self: 'static;
}

/// Validates that a physical connection exists between two matched wires in the haystack.
pub fn validate_connection<'ctx>(
    from: &CellId,
    to: &CellId,
    haystack_index: &GraphIndex<'ctx>,
) -> bool {
    // validate_connection_inner(from, to, haystack_index).unwrap_or(false)
    todo!();
}

/// Internal helper to resolve CellInfo to CellWrappers and check connectivity.
fn validate_connection_inner<'ctx>(
    from: &CellId,
    to: &CellId,
    haystack_index: &GraphIndex<'ctx>,
) -> Option<bool> {
    let from_id = from.cell_idx();
    let to_id = to.cell_idx();

    let f_wrapper = haystack_index.get_cell_by_id(from_id as usize)?;
    let t_wrapper = haystack_index.get_cell_by_id(to_id as usize)?;

    let fanout = haystack_index.fanout_set(f_wrapper)?;
    Some(fanout.contains(t_wrapper))
}
