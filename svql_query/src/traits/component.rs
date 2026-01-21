//! Unified component traits for structural pattern matching.
//!
//! This module provides the foundational traits that all query components
//! (Netlist, Composite, Variant) implement. Blanket implementations of
//! `Pattern` and `Matched` are provided for any type implementing these traits.

use std::any::TypeId;

use crate::prelude::*;
use crate::session::{
    ColumnDef, ExecutionContext, PatternRegistry, QueryError, Row, Store, Table,
};

/// Marker types for component kinds.
pub mod kind {
    /// Marker for netlist-based components (atomic patterns from HDL files).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Netlist;

    /// Marker for composite components (hierarchical patterns).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Composite;

    /// Marker for variant components (polymorphic patterns).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Variant;
}

/// Core trait for all searchable pattern components.
///
/// This trait unifies the behavior of Netlist, Composite, and Variant
/// components, enabling blanket implementations of `Pattern`.
///
/// # Type Parameters
/// - `Kind`: A marker type (`kind::Netlist`, `kind::Composite`, or `kind::Variant`)
///   that distinguishes the component category at the type level.
pub trait SearchableComponent: Hardware<State = Search> + Sized + Clone {
    /// Marker type identifying the component category.
    type Kind;

    /// The corresponding matched result type.
    type Match: MatchedComponent<Search = Self>;

    /// Creates a new search instance rooted at the given hierarchical path.
    fn create_at(base_path: Instance) -> Self;

    /// Builds the context required to execute this search.
    ///
    /// This typically involves loading needle designs and merging
    /// contexts from sub-components.
    ///
    /// **Legacy API** - Will be deprecated.
    fn build_context(
        driver: &Driver,
        options: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>>;

    /// Executes the search against the provided design context.
    ///
    /// **Legacy API** - Will be deprecated. Use `df_search()` instead.
    fn execute_search(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Match>;

    // =========================================================================
    // New DataFrame API (Phase 4)
    // =========================================================================

    /// Column schema for DataFrame storage.
    ///
    /// # Default Implementation
    /// Returns an empty slice.
    fn df_columns() -> &'static [ColumnDef] {
        &[]
    }

    /// Dependencies as TypeIds.
    ///
    /// # Default Implementation
    /// Returns an empty slice.
    fn df_dependencies() -> &'static [TypeId] {
        &[]
    }

    /// Register this component and all dependencies into the registry.
    ///
    /// # Default Implementation
    /// Registers self with no dependencies.
    fn df_register_all(registry: &mut PatternRegistry)
    where
        Self: 'static,
    {
        registry.register(
            TypeId::of::<Self>(),
            std::any::type_name::<Self>(),
            Self::df_dependencies(),
        );
    }

    /// Execute the search and return results as a Table.
    ///
    /// # Default Implementation
    /// Returns an empty table.
    fn df_search(_ctx: &ExecutionContext<'_>) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        Table::empty(Self::df_columns())
    }

    /// Rehydrate a Row back to the Match type-state.
    ///
    /// # Default Implementation
    /// Returns `None`.
    fn df_rehydrate(_row: &Row<Self>, _store: &Store) -> Option<Self::Match>
    where
        Self: 'static,
    {
        None
    }
}

/// Core trait for all matched result components.
///
/// Implemented by the `Match` state variants of query components.
pub trait MatchedComponent: Hardware<State = Match> + Sized + Clone {
    /// The search pattern type that produced this match.
    type Search: SearchableComponent<Match = Self>;
}

// ============================================================================
// Blanket Implementations
// ============================================================================

impl<T> Pattern for T
where
    T: SearchableComponent,
{
    type Match = T::Match;

    fn instantiate(base_path: Instance) -> Self {
        T::create_at(base_path)
    }

    fn context(
        driver: &Driver,
        options: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        T::build_context(driver, options)
    }

    fn execute(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Match> {
        self.execute_search(driver, context, key, config)
    }

    // New DataFrame API delegations

    fn columns() -> &'static [ColumnDef] {
        T::df_columns()
    }

    fn dependencies() -> &'static [TypeId] {
        T::df_dependencies()
    }

    fn register_all(registry: &mut PatternRegistry)
    where
        Self: 'static,
    {
        T::df_register_all(registry)
    }

    fn search(ctx: &ExecutionContext<'_>) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        T::df_search(ctx)
    }

    fn rehydrate(row: &Row<Self>, store: &Store) -> Option<Self::Match>
    where
        Self: 'static,
    {
        T::df_rehydrate(row, store)
    }
}

impl<T> Matched for T
where
    T: MatchedComponent,
{
    type Search = T::Search;
}
