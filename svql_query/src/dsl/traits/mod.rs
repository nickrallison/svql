//! Core traits for query components and execution.
//!
//! Defines the interfaces that all netlists, composites, and variants must
//! implement to participate in the SVQL query system.

/// Hierarchical grouping patterns.
pub mod composite;
pub mod display;
pub mod netlist;
pub mod primitive;
pub mod recursive;
/// Choice-based implementation patterns.
pub mod variant;

// Re-export key traits
pub use display::{ReportNode, wire_source_location, wire_to_report_node};
pub use netlist::Netlist;
pub use primitive::Primitive;
use tracing::info;

use crate::prelude::*;
use crate::session::EntryArray;

/// Apply automatic deduplication to a list of entries.
///
/// Removes rows with identical signatures (all wire references, submodule references,
/// and metadata). This is always enabled to prevent redundant results.
pub fn apply_deduplication(entries: &mut Vec<EntryArray>) {
    let mut seen = HashSet::new();
    entries.retain(|entry| seen.insert(entry.signature()));
}

/// Executes a search function and returns the result as a type-erased table.
///
/// # Errors
///
/// Returns a `QueryError` if the search fails.
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
    /// The raw column definitions.
    const DEFS: &'static [ColumnDef];

    /// Size of the schema (number of columns).
    const SCHEMA_SIZE: usize = Self::DEFS.len();

    /// Info needed to execute this pattern. Used to build the `ExecutionPlan` DAG.
    const EXEC_INFO: &'static ExecInfo;

    /// Access the smart Schema wrapper.
    ///
    /// Implementation should use a `static OnceLock<Schema>` to ensure
    /// the schema is built only once.
    fn schema() -> &'static PatternSchema;

    /// Get the indices of all Input columns in the Schema.
    #[must_use]
    fn input_indices() -> Vec<usize> {
        Self::schema().inputs.clone()
    }

    /// Get the indices of all Output columns in the Schema.
    #[must_use]
    fn output_indices() -> Vec<usize> {
        Self::schema().outputs.clone()
    }

    /// Check if a specific column name is an Output.
    #[must_use]
    fn is_output(name: &str) -> bool {
        Self::schema()
            .get(name)
            .is_some_and(|col| col.direction == PortDirection::Output)
    }

    /// Check if a specific column name is an Input.
    #[must_use]
    fn is_input(name: &str) -> bool {
        Self::schema()
            .get(name)
            .is_some_and(|col| col.direction == PortDirection::Input)
    }

    /// Returns the static type name of the component.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Loads designs into the driver cache before the search phase.
    ///
    /// # Errors
    ///
    /// Returns an error if design loading fails.
    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Execute the search and return results as a Table.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the search fails.
    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>;

    /// Execute the search and return results as a boxed `AnyTable`.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the search fails.
    fn search_table_any(
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>
    where
        Self: Component + 'static,
    {
        let table = Self::search_table(ctx)?;
        Ok(Box::new(table))
    }

    /// Entry point for plan execution.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the search fails.
    fn search(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<Store, QueryError> {
        info!("═══════════════════════════════════════════════════════");
        info!(
            "Starting pattern search for: {}",
            std::any::type_name::<Self>()
        );
        info!("Design: {:?}", design_key);
        info!(
            "Config: parallel={}, match_length={:?}",
            config.parallel, config.match_length
        );
        info!("═══════════════════════════════════════════════════════");

        let (plan, slots) = ExecutionPlan::build(Self::EXEC_INFO);

        info!(
            "Preloading driver for pattern: {}, haystack: {:?}",
            std::any::type_name::<Self>(),
            design_key
        );
        Self::preload_driver(driver, design_key, config)
            .map_err(|e| QueryError::design_load(e.to_string()))?;
        info!("Driver preload complete");

        info!(
            "Executing search for pattern: {}",
            std::any::type_name::<Self>()
        );
        let store = plan.execute(driver, design_key, config, slots)?;

        info!("═══════════════════════════════════════════════════════");
        info!(
            "Pattern search completed for: {}",
            std::any::type_name::<Self>()
        );
        info!("Store contains {} table(s)", store.len());
        info!("═══════════════════════════════════════════════════════\n");

        Ok(store)
    }

    /// Rehydrate a Row back to the Match type-state.
    ///
    /// Reconstructs a full `Self::Match` object from the row data and
    /// any referenced submodule rows in the store.
    fn rehydrate(row: &Row<Self>, store: &Store, driver: &Driver, key: &DriverKey) -> Option<Self>
    where
        Self: Component + 'static;

    /// Create a hierarchical report node from a match row.
    fn row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> ReportNode
    where
        Self: Component + 'static;

    /// Render a match row as a formatted string.
    fn render_row(row: &Row<Self>, store: &Store, driver: &Driver, key: &DriverKey) -> String
    where
        Self: Component + 'static,
    {
        Self::row_to_report_node(row, store, driver, key).render()
    }
}

/// Identifies the fundamental implementation style of a hardware component.
pub mod kind {
    /// Matches an external netlist file.
    pub struct Netlist;
    /// Matches a hierarchy of sub-patterns.
    pub struct Composite;
    /// Matches one of multiple possible implementations.
    pub struct Variant;
    /// Matches a direct cell kind in the design.
    pub struct Primitive;
    /// Matches a self-referential tree structure.
    pub struct Recursive;
}

/// Basic hardware component trait for kind-specification.
pub trait Component {
    /// The Kind struct from the kind module.
    type Kind;
}

/// Trait implemented by logic defining how to execute a pattern Kind.
pub trait PatternInternal<K>: Sized {
    /// Initial definition list.
    const DEFS: &'static [ColumnDef];
    /// Column count.
    const SCHEMA_SIZE: usize = Self::DEFS.len();
    /// Execution metadata.
    const EXEC_INFO: &'static crate::session::ExecInfo;

    /// Return the specific schema for this implementation.
    fn internal_schema() -> &'static crate::session::PatternSchema;

    /// Pre-fetch required designs into the driver cache.
    ///
    /// # Errors
    ///
    /// Returns an error if pre-loading fails.
    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Primary search logic for the pattern Kind.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the search fails.
    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static;

    /// Reconstructs the rust struct from a type-erased row.
    fn internal_rehydrate(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<Self::Kind> + Send + Sync + 'static;

    /// Generates a report node from stored row data.
    fn internal_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> ReportNode
    where
        Self: Component + PatternInternal<Self::Kind> + Send + Sync + 'static;
}

impl<T> Pattern for T
where
    T: Component + PatternInternal<T::Kind> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = T::DEFS;

    const SCHEMA_SIZE: usize = T::SCHEMA_SIZE;

    const EXEC_INFO: &'static crate::session::ExecInfo = T::EXEC_INFO;

    fn schema() -> &'static PatternSchema {
        T::internal_schema()
    }

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
        T::internal_rehydrate(row, store, driver, key)
    }

    fn row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> ReportNode {
        T::internal_row_to_report_node(row, store, driver, key)
    }
}
